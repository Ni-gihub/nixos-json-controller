use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub has_system_packages: bool,
    pub has_systemd_services: bool,
    pub has_service_options: bool,
    pub declared_packages: BTreeSet<String>,
    pub declared_services: BTreeSet<String>,
    pub imports: Vec<PathBuf>,
    pub write_safety: WriteSafety,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteSafety {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigState {
    pub files: Vec<ConfigFile>,
}

impl ConfigState {
    pub fn discover(flake_root: &Path) -> Result<Self, String> {
        let mut paths = Vec::new();
        collect_nix_files(flake_root, 0, &mut paths)?;

        let mut files = Vec::new();
        for path in paths {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
            files.push(inspect_file(path, &content));
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self { files })
    }

    pub fn package_declarations(&self, package: &str) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|file| file.declared_packages.contains(package))
            .map(|file| file.path.as_path())
            .collect()
    }

    pub fn service_declarations(&self, service: &str) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|file| file.declared_services.contains(service))
            .map(|file| file.path.as_path())
            .collect()
    }

    pub fn package_write_targets(&self) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|file| file.has_system_packages && file.write_safety == WriteSafety::Safe)
            .map(|file| file.path.as_path())
            .collect()
    }

    pub fn service_write_targets(&self) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|file| {
                (file.has_systemd_services || file.has_service_options)
                    && file.write_safety == WriteSafety::Safe
            })
            .map(|file| file.path.as_path())
            .collect()
    }

    pub fn file(&self, path: &Path) -> Option<&ConfigFile> {
        self.files.iter().find(|file| file.path == path)
    }

    pub fn is_safe_write_target(&self, path: &Path) -> bool {
        self.file(path)
            .is_some_and(|file| file.write_safety == WriteSafety::Safe)
    }
}

fn collect_nix_files(
    directory: &Path,
    depth: usize,
    paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Ok(());
    }

    let entries = fs::read_dir(directory)
        .map_err(|e| format!("failed to read {}: {}", directory.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
            continue;
        }

        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| format!("failed to inspect {}: {}", path.display(), e))?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.file_type().is_dir() {
            collect_nix_files(&path, depth + 1, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("nix") {
            paths.push(path);
        }
    }

    Ok(())
}

fn inspect_file(path: PathBuf, content: &str) -> ConfigFile {
    let declared_packages = parse_system_package_names(content).into_iter().collect();
    let declared_services = parse_service_names(content).into_iter().collect();

    let imports = parse_relative_imports(path.parent().unwrap_or_else(|| Path::new(".")), content);

    let write_safety = classify_write_safety(&path, content, &imports);

    ConfigFile {
        path,
        has_system_packages: contains_assignment(content, "environment.systemPackages"),
        has_systemd_services: content.contains("systemd.services."),
        has_service_options: content.contains("services.") && content.contains(".enable"),
        declared_packages,
        declared_services,
        write_safety,
        imports,
    }
}

fn classify_write_safety(_path: &Path, content: &str, _imports: &[PathBuf]) -> WriteSafety {
    // Imports and module-system combinators are normal NixOS module composition.
    // They do not by themselves make a file unsafe to edit.
    //
    // The safety boundary here is intentionally narrow: reject files that mix
    // multiple declarations of the same target option, or files that clearly
    // belong to another module namespace such as Home Manager. The caller can
    // then use declaration/provenance information to decide where an actual
    // write should happen instead of relying on filenames or import structure.
    let lower = content.to_ascii_lowercase();
    let unsafe_markers = ["home-manager", "home.packages", "home.activation"];

    if unsafe_markers.iter().any(|marker| lower.contains(marker)) {
        return WriteSafety::Unsafe;
    }

    let package_assignments = count_assignment(content, "environment.systemPackages");
    let service_assignments = count_service_enable_assignments(content);

    if package_assignments > 1 || service_assignments > 1 {
        return WriteSafety::Unsafe;
    }

    WriteSafety::Safe
}

fn count_assignment(content: &str, attribute: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let code = strip_comment(line);
            code.contains(attribute) && code.contains('=')
        })
        .count()
}

fn count_service_enable_assignments(content: &str) -> usize {
    ["systemd.services.", "services."]
        .iter()
        .map(|prefix| {
            content
                .lines()
                .filter(|line| {
                    let code = strip_comment(line);
                    code.contains(prefix) && code.contains(".enable") && code.contains('=')
                })
                .count()
        })
        .sum()
}

fn contains_assignment(content: &str, attribute: &str) -> bool {
    content.lines().any(|line| {
        let code = strip_comment(line);
        code.contains(attribute) && code.contains('=')
    })
}

fn parse_system_package_names(content: &str) -> Vec<String> {
    let Some(start) = find_system_packages_list(content) else {
        return Vec::new();
    };
    let Some(end) = find_matching_delimiter(content, start, '[', ']') else {
        return Vec::new();
    };

    content[start + 1..end]
        .lines()
        .filter_map(parse_package_line)
        .collect()
}

fn find_system_packages_list(content: &str) -> Option<usize> {
    let marker = "environment.systemPackages";
    let mut offset = 0usize;

    for line in content.split_inclusive('\n') {
        let code = strip_comment(line);
        if let Some(position) = code.find(marker) {
            let marker_pos = offset + position;
            let assignment = content[marker_pos..].find('=')? + marker_pos;
            return content[assignment + 1..]
                .find('[')
                .map(|offset| assignment + 1 + offset);
        }
        offset += line.len();
    }

    if offset < content.len() {
        let code = strip_comment(&content[offset..]);
        if let Some(position) = code.find(marker) {
            let marker_pos = offset + position;
            let assignment = content[marker_pos..].find('=')? + marker_pos;
            return content[assignment + 1..]
                .find('[')
                .map(|offset| assignment + 1 + offset);
        }
    }

    None
}

fn parse_package_line(line: &str) -> Option<String> {
    let line = strip_comment(line).trim();
    if line.is_empty() || line.contains('=') || line.contains(';') {
        return None;
    }

    let token = line
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| matches!(c, '[' | ']' | '(' | ')' | ','));
    let token = token.strip_prefix("pkgs.").unwrap_or(token);

    if token.is_empty() || token.contains('.') {
        return None;
    }

    token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        .then(|| token.to_string())
}

fn parse_service_names(content: &str) -> Vec<String> {
    let mut services = BTreeSet::new();

    for line in content.lines() {
        let line = strip_comment(line);
        for prefix in ["systemd.services.", "services."] {
            let Some((_, rest)) = line.split_once(prefix) else {
                continue;
            };
            let Some((service, suffix)) = rest.split_once(".enable") else {
                continue;
            };

            if !service.is_empty()
                && suffix.trim_start().starts_with('=')
                && service
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                services.insert(service.to_string());
            }
        }
    }

    services.into_iter().collect()
}

fn parse_relative_imports(base: &Path, content: &str) -> Vec<PathBuf> {
    let mut imports = Vec::new();
    let mut in_imports = false;

    for line in content.lines() {
        let line = strip_comment(line);
        if !in_imports {
            let Some((_, rest)) = line.split_once("imports") else {
                continue;
            };
            if !rest.contains('=') {
                continue;
            }
            in_imports = true;
        }

        let source = line;
        for token in source.split_whitespace() {
            let token = token.trim_matches(|c: char| matches!(c, '[' | ']' | ';' | ','));
            if (token.starts_with("./") || token.starts_with("../")) && token.ends_with(".nix") {
                imports.push(normalize_relative_path(base, token));
            }
        }

        if in_imports && line.contains(']') {
            in_imports = false;
        }
    }

    imports.sort();
    imports.dedup();
    imports
}

fn normalize_relative_path(base: &Path, relative: &str) -> PathBuf {
    let mut path = PathBuf::from(base);

    for component in Path::new(relative).components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !path.pop() {
                    path.push("..");
                }
            }
            std::path::Component::Normal(component) => path.push(component),
            _ => {}
        }
    }

    path
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(code, _)| code)
}

fn find_matching_delimiter(content: &str, start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;

    for (offset, ch) in content[start..].char_indices() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '#' => in_comment = true,
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspects_package_and_service_declarations() {
        let file = inspect_file(
            PathBuf::from("modules/core.nix"),
            r#"
environment.systemPackages = with pkgs; [
  firefox
  pkgs.git
];

systemd.services.sshd.enable = true;
services.openssh.enable = true;
"#,
        );

        assert!(file.has_system_packages);
        assert!(file.has_systemd_services);
        assert!(file.has_service_options);
        assert!(file.declared_packages.contains("firefox"));
        assert!(file.declared_packages.contains("git"));
        assert!(file.declared_services.contains("sshd"));
        assert!(file.declared_services.contains("openssh"));
    }

    #[test]
    fn ignores_variables_but_recognizes_real_service_options() {
        let file = inspect_file(
            PathBuf::from("configuration.nix"),
            r#"
let
  firefox = something;
in
{
  # environment.systemPackages = [ fake ];
  services.xserver.enable = true;
}
"#,
        );

        assert!(file.declared_packages.is_empty());
        assert!(file.declared_services.contains("xserver"));
    }

    #[test]
    fn parses_nested_package_expressions() {
        let file = inspect_file(
            PathBuf::from("packages.nix"),
            r#"
environment.systemPackages = lib.mkAfter [
  pkgs.git
  pkgs.firefox
];
"#,
        );

        assert!(file.declared_packages.contains("git"));
        assert!(file.declared_packages.contains("firefox"));
    }

    #[test]
    fn finds_imports() {
        let file = inspect_file(
            PathBuf::from("/tmp/configuration.nix"),
            r#"
{
  imports = [
    ./hardware-configuration.nix
    ./modules/desktop.nix
  ];
}
"#,
        );

        assert_eq!(
            file.imports,
            vec![
                PathBuf::from("/tmp/hardware-configuration.nix"),
                PathBuf::from("/tmp/modules/desktop.nix")
            ]
        );
    }

    #[test]
    fn rejects_home_manager_configuration() {
        let file = inspect_file(
            PathBuf::from("home.nix"),
            r#"
home.packages = [ pkgs.firefox ];
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Unsafe);
    }

    #[test]
    fn accepts_module_wrappers() {
        let file = inspect_file(
            PathBuf::from("modules/packages.nix"),
            r#"
{
  environment.systemPackages = lib.mkMerge [
    [ pkgs.firefox ]
    (lib.mkIf config.foo [ pkgs.git ])
  ];
}
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Safe);
    }

    #[test]
    fn accepts_importing_aggregator() {
        let file = inspect_file(
            PathBuf::from("hosts/laptop/default.nix"),
            r#"
{
  imports = [ ./hardware.nix ];
  environment.systemPackages = [ pkgs.git ];
}
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Safe);
    }

    #[test]
    fn accepts_leaf_module_with_unambiguous_target() {
        let file = inspect_file(
            PathBuf::from("modules/browser/firefox.nix"),
            r#"
{
  environment.systemPackages = [ pkgs.firefox ];
}
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Safe);
    }

    #[test]
    fn rejects_multiple_package_declarations() {
        let file = inspect_file(
            PathBuf::from("modules/packages.nix"),
            r#"
{
  environment.systemPackages = [ pkgs.firefox ];
  environment.systemPackages = lib.mkAfter [ pkgs.git ];
}
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Unsafe);
    }

    #[test]
    fn accepts_conventional_aggregate_file() {
        let file = inspect_file(
            PathBuf::from("packages.nix"),
            r#"
{
  environment.systemPackages = [ pkgs.firefox pkgs.git ];
}
"#,
        );

        assert_eq!(file.write_safety, WriteSafety::Safe);
    }

    #[test]
    fn finds_matching_brackets() {
        let content = r#"environment.systemPackages = [ pkgs.git (foo "[]") ];"#;
        let start = find_system_packages_list(content).unwrap();
        let end = find_matching_delimiter(content, start, '[', ']').unwrap();

        assert_eq!(&content[start..=end], r#"[ pkgs.git (foo "[]") ]"#);
    }
}
