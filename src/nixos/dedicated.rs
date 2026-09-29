use std::fs;
use std::path::{Path, PathBuf};

use super::config::ConfigState;
use super::module::find_matching_delimiter;
use super::writer;

const DEDICATED_PACKAGE_MODULE: &str = "nxc/packages.nix";
const DEDICATED_SERVICE_MODULE: &str = "nxc/services.nix";

#[derive(Debug, Clone)]
pub struct FileBackup {
    pub path: PathBuf,
    pub content: String,
}

impl FileBackup {
    pub fn capture(path: &Path) -> Result<Self, String> {
        Ok(Self {
            path: path.to_path_buf(),
            content: fs::read_to_string(path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?,
        })
    }

    pub fn restore(&self) -> Result<(), String> {
        fs::write(&self.path, &self.content)
            .map_err(|e| format!("failed to restore {}: {}", self.path.display(), e))
    }
}

#[derive(Debug, Clone, Default)]
pub struct DedicatedPackageChange {
    pub backups: Vec<FileBackup>,
    pub created_files: Vec<PathBuf>,
}

impl DedicatedPackageChange {
    pub fn rollback(&self) -> Result<(), String> {
        for backup in &self.backups {
            backup.restore()?;
        }

        for path in &self.created_files {
            if path.is_file() {
                fs::remove_file(path)
                    .map_err(|e| format!("failed to remove {}: {}", path.display(), e))?;
            }
        }

        Ok(())
    }
}

pub fn install_package(
    flake_root: &Path,
    config: &ConfigState,
    package: &str,
) -> Result<DedicatedPackageChange, String> {
    let module_path = flake_root.join(DEDICATED_PACKAGE_MODULE);
    let mut change = DedicatedPackageChange::default();

    if module_path.is_file() {
        change.backups.push(FileBackup::capture(&module_path)?);
    } else {
        let parent = module_path
            .parent()
            .ok_or_else(|| format!("invalid dedicated module path: {}", module_path.display()))?;

        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;

        fs::write(
            &module_path,
            "{ pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
  ];
}
",
        )
        .map_err(|e| format!("failed to create {}: {}", module_path.display(), e))?;

        change.created_files.push(module_path.clone());
    }

    if let Err(error) = ensure_import(flake_root, config, &module_path, &mut change) {
        let _ = change.rollback();
        return Err(error);
    }

    if let Err(error) = writer::install_package(&module_path, package) {
        let _ = change.rollback();
        return Err(error);
    }

    Ok(change)
}

#[derive(Debug, Clone, Default)]
pub struct DedicatedServiceChange {
    pub backups: Vec<FileBackup>,
    pub created_files: Vec<PathBuf>,
}

impl DedicatedServiceChange {
    pub fn rollback(&self) -> Result<(), String> {
        for backup in &self.backups {
            backup.restore()?;
        }

        for path in &self.created_files {
            if path.is_file() {
                fs::remove_file(path)
                    .map_err(|e| format!("failed to remove {}: {}", path.display(), e))?;
            }
        }

        Ok(())
    }
}

pub fn enable_service(
    flake_root: &Path,
    config: &ConfigState,
    service: &str,
) -> Result<DedicatedServiceChange, String> {
    write_service(flake_root, config, service, true)
}

pub fn disable_service(
    flake_root: &Path,
    config: &ConfigState,
    service: &str,
) -> Result<DedicatedServiceChange, String> {
    write_service(flake_root, config, service, false)
}

fn write_service(
    flake_root: &Path,
    config: &ConfigState,
    service: &str,
    enabled: bool,
) -> Result<DedicatedServiceChange, String> {
    let module_path = flake_root.join(DEDICATED_SERVICE_MODULE);
    let mut change = DedicatedServiceChange::default();

    if module_path.is_file() {
        change.backups.push(FileBackup::capture(&module_path)?);
    } else {
        let parent = module_path
            .parent()
            .ok_or_else(|| format!("invalid dedicated module path: {}", module_path.display()))?;

        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;

        fs::write(&module_path, "{ ... }:\n\n{\n}\n")
            .map_err(|e| format!("failed to create {}: {}", module_path.display(), e))?;

        change.created_files.push(module_path.clone());
    }

    if let Err(error) = ensure_import(flake_root, config, &module_path, &mut change.backups) {
        let _ = change.rollback();
        return Err(error);
    }

    if let Err(error) = if enabled {
        writer::enable_service(&module_path, service)
    } else {
        writer::disable_service(&module_path, service)
    } {
        let _ = change.rollback();
        return Err(error);
    }

    Ok(change)
}


fn ensure_import(
    flake_root: &Path,
    config: &ConfigState,
    module_path: &Path,
    backups: &mut Vec<FileBackup>,
) -> Result<(), String> {
    if find_importing_file(flake_root, config, module_path)?.is_some() {
        return Ok(());
    }

    let candidates = import_targets(flake_root, config)?;
    let target = match candidates.as_slice() {
        [target] => target.clone(),
        [] => {
            return Err(
                "cannot connect NXC dedicated module: no unambiguous imports list was found"
                    .to_string(),
            )
        }
        _ => {
            return Err(format!(
                "cannot connect NXC dedicated module: multiple imports lists were found: {}",
                format_paths(&candidates),
            ))
        }
    };

    let backup = FileBackup::capture(&target)?;
    let content = backup.content.clone();
    let import_path = relative_import_path(&target, module_path)?;

    let updated = add_import_to_content(&content, &import_path)?;
    if updated == content {
        return Ok(());
    }

    fs::write(&target, updated)
        .map_err(|e| format!("failed to update imports in {}: {}", target.display(), e))?;

    backups.push(backup);
    Ok(())
}

fn find_importing_file(
    flake_root: &Path,
    config: &ConfigState,
    module_path: &Path,
) -> Result<Option<PathBuf>, String> {
    let module_path = fs::canonicalize(module_path)
        .map_err(|e| format!("failed to resolve {}: {}", module_path.display(), e))?;

    for file in &config.files {
        let content = fs::read_to_string(&file.path)
            .map_err(|e| format!("failed to read {}: {}", file.path.display(), e))?;

        let Some(parent) = file.path.parent() else {
            continue;
        };

        for import in &file.imports {
            if fs::canonicalize(import).ok().as_ref() == Some(&module_path) {
                return Ok(Some(file.path.clone()));
            }

            let candidate = parent.join(import);
            if fs::canonicalize(candidate).ok().as_ref() == Some(&module_path) {
                return Ok(Some(file.path.clone()));
            }
        }

        if content.contains(&relative_import_path(&file.path, &module_path)?) {
            return Ok(Some(file.path.clone()));
        }
    }

    let flake_file = flake_root.join("flake.nix");
    if flake_file.is_file() {
        let content = fs::read_to_string(&flake_file)
            .map_err(|e| format!("failed to read {}: {}", flake_file.display(), e))?;
        if content.contains(&relative_import_path(&flake_file, &module_path)?) {
            return Ok(Some(flake_file));
        }
    }

    Ok(None)
}

fn import_targets(flake_root: &Path, config: &ConfigState) -> Result<Vec<PathBuf>, String> {
    let mut targets = Vec::new();

    for file in &config.files {
        let content = fs::read_to_string(&file.path)
            .map_err(|e| format!("failed to read {}: {}", file.path.display(), e))?;

        if find_import_list(content.as_str()).is_some() {
            targets.push(file.path.clone());
        }
    }

    let flake_file = flake_root.join("flake.nix");
    if flake_file.is_file() {
        let content = fs::read_to_string(&flake_file)
            .map_err(|e| format!("failed to read {}: {}", flake_file.display(), e))?;

        if find_import_list(&content).is_some() {
            targets.push(flake_file);
        }
    }

    targets.sort();
    targets.dedup();
    Ok(targets)
}

fn find_import_list(content: &str) -> Option<usize> {
    let marker = "imports";
    let marker_position = content.find(marker)?;
    let assignment = content[marker_position..].find('=')? + marker_position;
    let list = content[assignment + 1..].find('[')? + assignment + 1;
    find_matching_delimiter(content, list, '[', ']')?;
    Some(list)
}

fn add_import_to_content(content: &str, import_path: &str) -> Result<String, String> {
    let list_start = find_import_list(content)
        .ok_or_else(|| "imports list not found".to_string())?;
    let insert_at = list_start + 1;

    let indentation = content[list_start..]
        .lines()
        .nth(1)
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).collect::<String>())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "  ".to_string());

    let entry = format!("
{}{}", indentation, import_path);

    let mut result = content.to_string();
    result.insert_str(insert_at, &entry);
    Ok(result)
}

fn relative_import_path(from_file: &Path, target: &Path) -> Result<String, String> {
    let from_dir = from_file
        .parent()
        .ok_or_else(|| format!("invalid import source: {}", from_file.display()))?;

    let from = fs::canonicalize(from_dir)
        .map_err(|e| format!("failed to resolve {}: {}", from_dir.display(), e))?;
    let target = fs::canonicalize(target)
        .map_err(|e| format!("failed to resolve {}: {}", target.display(), e))?;

    let from_components: Vec<_> = from.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let common = from_components
        .iter()
        .zip(&target_components)
        .take_while(|(a, b)| a == b)
        .count();

    let mut parts = Vec::new();
    for _ in common..from_components.len() {
        parts.push("..".to_string());
    }

    for component in &target_components[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }

    let path = parts.join("/");
    Ok(if path.starts_with('.') {
        path
    } else {
        format!("./{}", path)
    })
}

fn format_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_import_to_existing_list() {
        let content = r#"{ imports = [
  ./hardware.nix
]; }"#;

        let updated = add_import_to_content(content, "./nxc/packages.nix").unwrap();

        assert!(updated.contains("./nxc/packages.nix"));
        assert!(updated.contains("./hardware.nix"));
    }

    #[test]
    fn finds_import_list() {
        let content = r#"{ imports = [ ./hardware.nix ]; }"#;
        assert!(find_import_list(content).is_some());
    }
}
