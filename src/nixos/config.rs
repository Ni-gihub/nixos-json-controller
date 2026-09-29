use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub has_system_packages: bool,
    pub has_systemd_services: bool,
    pub declared_packages: BTreeSet<String>,
    pub declared_services: BTreeSet<String>,
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
            .filter(|file| file.has_system_packages)
            .map(|file| file.path.as_path())
            .collect()
    }

    pub fn service_write_targets(&self) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|file| file.has_systemd_services)
            .map(|file| file.path.as_path())
            .collect()
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
        let entry = entry
            .map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
            continue;
        }

        if path.is_dir() {
            collect_nix_files(&path, depth + 1, paths)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("nix") {
            paths.push(path);
        }
    }

    Ok(())
}

fn inspect_file(path: PathBuf, content: &str) -> ConfigFile {
    let mut declared_packages = BTreeSet::new();
    let mut declared_services = BTreeSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(package) = parse_package_line(trimmed) {
            declared_packages.insert(package);
        }

        if let Some(service) = parse_service_line(trimmed) {
            declared_services.insert(service);
        }
    }

    ConfigFile {
        path,
        has_system_packages: content.contains("environment.systemPackages"),
        has_systemd_services: content.contains("systemd.services."),
        declared_packages,
        declared_services,
    }
}

fn parse_package_line(line: &str) -> Option<String> {
    if line.is_empty()
        || line.starts_with('#')
        || line.contains('=')
        || line.contains('[')
        || line.contains(']')
        || line.contains(';')
    {
        return None;
    }

    let token = line.split_whitespace().next()?;

    if token.starts_with("pkgs.") {
        return Some(token.trim_start_matches("pkgs.").to_string());
    }

    if token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Some(token.to_string());
    }

    None
}

fn parse_service_line(line: &str) -> Option<String> {
    let prefix = "systemd.services.";
    let rest = line.strip_prefix(prefix)?;
    let (service, _) = rest.split_once(".enable")?;

    if service.is_empty()
        || !service
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }

    Some(service.to_string())
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
"#,
        );

        assert!(file.has_system_packages);
        assert!(file.has_systemd_services);
        assert!(file.declared_packages.contains("firefox"));
        assert!(file.declared_packages.contains("git"));
        assert!(file.declared_services.contains("sshd"));
    }

    #[test]
    fn ignores_comments_and_option_lines_as_packages() {
        let file = inspect_file(
            PathBuf::from("configuration.nix"),
            r#"
# firefox
environment.systemPackages = with pkgs; [
  firefox
  foo = bar;
];
"#,
        );

        assert!(file.declared_packages.contains("firefox"));
        assert!(!file.declared_packages.contains("environment.systemPackages"));
        assert!(!file.declared_packages.contains("foo"));
    }

    #[test]
    fn finds_package_and_service_targets() {
        let state = ConfigState {
            files: vec![
                ConfigFile {
                    path: PathBuf::from("packages.nix"),
                    has_system_packages: true,
                    has_systemd_services: false,
                    declared_packages: BTreeSet::from(["firefox".to_string()]),
                    declared_services: BTreeSet::new(),
                },
                ConfigFile {
                    path: PathBuf::from("services.nix"),
                    has_system_packages: false,
                    has_systemd_services: true,
                    declared_packages: BTreeSet::new(),
                    declared_services: BTreeSet::from(["sshd".to_string()]),
                },
            ],
        };

        assert_eq!(
            state.package_declarations("firefox"),
            vec![Path::new("packages.nix")]
        );
        assert_eq!(
            state.service_declarations("sshd"),
            vec![Path::new("services.nix")]
        );
        assert_eq!(
            state.package_write_targets(),
            vec![Path::new("packages.nix")]
        );
    }
}
