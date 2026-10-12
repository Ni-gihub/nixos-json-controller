use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::config::ConfigState;
use super::module::find_matching_delimiter;
use super::writer;

pub const DEDICATED_PACKAGE_MODULE: &str = "nxc/packages.nix";
pub const DEDICATED_SERVICE_MODULE: &str = "nxc/services.nix";

const DEDICATED_PACKAGE_MARKER: &str = "environment.systemPackages = with pkgs;";
const DEDICATED_SERVICE_MARKER: &str = "{ ... }:";

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
        writer::write_atomic(&self.path, &self.content)
            .map_err(|e| format!("failed to restore {}: {}", self.path.display(), e))
    }
}

pub fn combine_operation_and_rollback_error(
    operation_error: String,
    rollback_result: Result<(), String>,
) -> String {
    match rollback_result {
        Ok(()) => operation_error,
        Err(rollback_error) => {
            format!("{}; rollback also failed: {}", operation_error, rollback_error)
        }
    }
}

/// Restore every part of a change, even when one restoration operation fails.
pub fn rollback_changes(
    staged_files: &[StagedFile],
    backups: &[FileBackup],
    created_files: &[PathBuf],
) -> Result<(), String> {
    let mut errors = Vec::new();

    for staged in staged_files.iter().rev() {
        if let Err(error) = restore_index_state(staged) {
            errors.push(error);
        }
    }

    for backup in backups.iter().rev() {
        if let Err(error) = backup.restore() {
            errors.push(error);
        }
    }

    for path in created_files.iter().rev() {
        if path.is_file() {
            if let Err(error) = fs::remove_file(path) {
                errors.push(format!("failed to remove {}: {}", path.display(), error));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Validate and back up all target files before applying any mutation.
pub fn apply_file_changes<V, M>(
    paths: &[PathBuf],
    mut validate: V,
    mut mutate: M,
) -> Result<Vec<FileBackup>, String>
where
    V: FnMut(&Path) -> Result<(), String>,
    M: FnMut(&Path) -> Result<(), String>,
{
    for path in paths {
        validate(path)?;
    }

    let backups = paths
        .iter()
        .map(|path| FileBackup::capture(path))
        .collect::<Result<Vec<_>, _>>()?;

    for path in paths {
        if let Err(error) = mutate(path) {
            return Err(combine_operation_and_rollback_error(
                error,
                rollback_changes(&[], &backups, &[]),
            ));
        }
    }

    Ok(backups)
}

#[derive(Debug, Clone, Default)]
pub struct DedicatedPackageChange {
    pub backups: Vec<FileBackup>,
    pub created_files: Vec<PathBuf>,
    pub staged_files: Vec<StagedFile>,
}

impl DedicatedPackageChange {
    pub fn rollback(&self) -> Result<(), String> {
        rollback_changes(&self.staged_files, &self.backups, &self.created_files)
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
        validate_dedicated_package_module(&module_path)?;
    } else {
        let parent = module_path
            .parent()
            .ok_or_else(|| format!("invalid dedicated module path: {}", module_path.display()))?;

        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;

        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&module_path)
        {
            Ok(file) => file,
            Err(error) => {
                return Err(combine_operation_and_rollback_error(
                    format!("failed to create {}: {}", module_path.display(), error),
                    change.rollback(),
                ));
            }
        };
        change.created_files.push(module_path.clone());
        if let Err(error) = file.write_all(
            b"{ pkgs, ... }:\n\n{\n  environment.systemPackages = with pkgs; [\n  ];\n}\n",
        ) {
            return Err(combine_operation_and_rollback_error(
                format!("failed to write {}: {}", module_path.display(), error),
                change.rollback(),
            ));
        }
    }

    if let Err(error) = ensure_import(flake_root, config, &module_path, &mut change.backups) {
        return Err(combine_operation_and_rollback_error(error, change.rollback()));
    }

    if let Err(error) = writer::install_package(&module_path, package) {
        return Err(combine_operation_and_rollback_error(error, change.rollback()));
    }

    for path in change.created_files.clone() {
        let staged = match stage_new_file(flake_root, &path) {
            Ok(Some(staged)) => staged,
            Ok(None) => continue,
            Err(error) => {
                return Err(combine_operation_and_rollback_error(error, change.rollback()));
            }
        };
        change.staged_files.push(staged);
    }

    Ok(change)
}

#[derive(Debug, Clone, Default)]
pub struct DedicatedServiceChange {
    pub backups: Vec<FileBackup>,
    pub created_files: Vec<PathBuf>,
    pub staged_files: Vec<StagedFile>,
}

impl DedicatedServiceChange {
    pub fn rollback(&self) -> Result<(), String> {
        rollback_changes(&self.staged_files, &self.backups, &self.created_files)
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
        validate_dedicated_service_module(&module_path)?;
    } else {
        let parent = module_path
            .parent()
            .ok_or_else(|| format!("invalid dedicated module path: {}", module_path.display()))?;

        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;

        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&module_path)
        {
            Ok(file) => file,
            Err(error) => {
                return Err(combine_operation_and_rollback_error(
                    format!("failed to create {}: {}", module_path.display(), error),
                    change.rollback(),
                ));
            }
        };
        change.created_files.push(module_path.clone());
        if let Err(error) = file.write_all(b"{ ... }:\n\n{\n}\n") {
            return Err(combine_operation_and_rollback_error(
                format!("failed to write {}: {}", module_path.display(), error),
                change.rollback(),
            ));
        }
    }

    if let Err(error) = ensure_import(flake_root, config, &module_path, &mut change.backups) {
        return Err(combine_operation_and_rollback_error(error, change.rollback()));
    }

    if let Err(error) = if enabled {
        writer::enable_service(&module_path, service)
    } else {
        writer::disable_service(&module_path, service)
    } {
        return Err(combine_operation_and_rollback_error(error, change.rollback()));
    }

    for path in change.created_files.clone() {
        let staged = match stage_new_file(flake_root, &path) {
            Ok(Some(staged)) => staged,
            Ok(None) => continue,
            Err(error) => {
                return Err(combine_operation_and_rollback_error(error, change.rollback()));
            }
        };
        change.staged_files.push(staged);
    }

    Ok(change)
}

fn validate_dedicated_package_module(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    if !content.contains(DEDICATED_PACKAGE_MARKER) {
        return Err(format!(
            "existing dedicated package module is not in the expected NXC format: {}",
            path.display()
        ));
    }

    Ok(())
}

fn validate_dedicated_service_module(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    if !content.contains(DEDICATED_SERVICE_MARKER) {
        return Err(format!(
            "existing dedicated service module is not in the expected NXC format: {}",
            path.display()
        ));
    }

    Ok(())
}

pub fn dedicated_package_path(flake_root: &Path) -> PathBuf {
    flake_root.join(DEDICATED_PACKAGE_MODULE)
}

pub fn dedicated_service_path(flake_root: &Path) -> PathBuf {
    flake_root.join(DEDICATED_SERVICE_MODULE)
}

#[derive(Debug, Clone)]
pub struct StagedFile {
    pub path: PathBuf,
    pub previous_index: Option<String>,
}

/// Stage only files newly created by NXC so Git-backed flakes can see them during evaluation.
pub fn stage_new_file(flake_root: &Path, path: &Path) -> Result<Option<StagedFile>, String> {
    let output = match Command::new("git")
        .args(["-C", flake_root.to_string_lossy().as_ref(), "rev-parse", "--is-inside-work-tree"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };

    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "true" {
        return Ok(None);
    }

    let relative = path.strip_prefix(flake_root).map_err(|e| {
        format!("failed to determine Git path for {}: {}", path.display(), e)
    })?;

    let index_state = Command::new("git")
        .args(["-C", flake_root.to_string_lossy().as_ref(), "ls-files", "--stage", "--"])
        .arg(relative)
        .output()
        .map_err(|e| format!("failed to inspect Git index for {}: {}", path.display(), e))?;

    if !index_state.status.success() {
        return Err(format!("failed to inspect Git index for {}", path.display()));
    }

    let previous_index = String::from_utf8_lossy(&index_state.stdout).trim().to_string();
    let previous_index = (!previous_index.is_empty()).then_some(previous_index);

    let staged = StagedFile {
        path: path.to_path_buf(),
        previous_index,
    };

    match Command::new("git")
        .args(["-C", flake_root.to_string_lossy().as_ref(), "add", "--"])
        .arg(relative)
        .status()
    {
        Ok(status) if status.success() => Ok(Some(staged)),
        // git add updates the index atomically: a failed command leaves the old
        // index in place, so do not try to restore it while index.lock may exist.
        Ok(_) => Err(format!(
            "failed to stage newly created NXC file: {}",
            path.display()
        )),
        Err(error) => Err(format!("failed to stage {}: {}", path.display(), error)),
    }
}

pub fn restore_index_state(staged: &StagedFile) -> Result<(), String> {
    let parent = staged.path.parent().ok_or_else(|| format!("invalid file path: {}", staged.path.display()))?;

    if let Some(index_entry) = &staged.previous_index {
        let mut child = Command::new("git")
            .args(["-C", parent.to_string_lossy().as_ref(), "update-index", "--index-info"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to start Git index restore for {}: {}", staged.path.display(), e))?;

        use std::io::Write;
        let write_result = match child.stdin.as_mut() {
            Some(stdin) => stdin
                .write_all(format!("{index_entry}\n").as_bytes())
                .map_err(|e| format!("failed to restore Git index for {}: {}", staged.path.display(), e)),
            None => Err("failed to open git stdin".to_string()),
        };
        drop(child.stdin.take());

        let status = child.wait()
            .map_err(|e| format!("failed to restore Git index for {}: {}", staged.path.display(), e))?;
        write_result?;

        if !status.success() {
            return Err(format!("failed to restore Git index for {}", staged.path.display()));
        }
    } else {
        let file_name = staged.path.file_name()
            .ok_or_else(|| format!("invalid file path: {}", staged.path.display()))?;

        let status = Command::new("git")
            .args(["-C", parent.to_string_lossy().as_ref(), "rm", "--cached", "--ignore-unmatch", "--"])
            .arg(file_name)
            .status()
            .map_err(|e| format!("failed to remove {} from Git index: {}", staged.path.display(), e))?;

        if !status.success() {
            return Err(format!("failed to restore Git index for {}", staged.path.display()));
        }
    }

    Ok(())
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

    let target = find_import_target(config)?;

    let backup = FileBackup::capture(&target)?;
    let content = backup.content.clone();
    let import_path = relative_import_path(&target, module_path)?;

    let updated = add_import_to_content(&content, &import_path)?;
    if updated == content {
        return Ok(());
    }

    backups.push(backup);
    writer::write_atomic(&target, &updated)
        .map_err(|e| format!("failed to update imports in {}: {}", target.display(), e))?;
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

/// Select the NixOS module that should connect a newly-created NXC module.
///
/// The configuration files form an import graph. A file that is not imported by
/// another discovered Nix file is a graph root and is therefore a better
/// connection point than an arbitrary file that happens to contain an imports list.
///
/// Conventional NixOS layouts get an additional preference: a unique
/// hosts/<name>/default.nix root is preferred, followed by configuration.nix
/// and a generic default.nix. This makes a layout such as
/// hosts/laptop/default.nix -> ../common.nix -> ../modules/*.nix resolve to
/// the host entrypoint instead of common.nix or hardware configuration.
fn find_import_target(config: &ConfigState) -> Result<PathBuf, String> {
    let imported_files = config
        .files
        .iter()
        .flat_map(|file| file.imports.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();

    let mut roots = config
        .files
        .iter()
        .filter(|file| file.path.file_name().and_then(|name| name.to_str()) != Some("flake.nix"))
        .filter(|file| !imported_files.contains(&file.path))
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();

    roots.sort();

    if roots.is_empty() {
        return Err("cannot connect NXC dedicated module: no NixOS module graph root was found".to_string());
    }

    let ranked = roots
        .iter()
        .map(|path| (root_score(path), path))
        .collect::<Vec<_>>();

    let best_score = ranked.iter().map(|(score, _)| *score).max().unwrap_or(0);
    let best = if best_score >= 60 {
        ranked
            .into_iter()
            .filter(|(score, _)| *score == best_score)
            .map(|(_, path)| path.clone())
            .collect::<Vec<_>>()
    } else if roots.len() == 1 {
        // A non-conventional layout is still valid when the discovered import
        // graph has a single unambiguous root. Do not require a particular
        // filename or directory structure to connect the dedicated module.
        roots
    } else {
        return Err(format!(
            "cannot connect NXC dedicated module: multiple non-conventional NixOS module graph roots found: {}",
            format_paths(&roots),
        ));
    };

    match best.as_slice() {
        [target] => Ok(target.clone()),
        _ => Err(format!(
            "cannot connect NXC dedicated module: multiple NixOS module graph roots found: {}",
            format_paths(&best),
        )),
    }
}

fn root_score(path: &Path) -> u8 {
    let file_name = path.file_name().and_then(|name| name.to_str());
    let parent_name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str());
    let grandparent_name = path
        .parent()
        .and_then(|parent| parent.parent())
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str());

    if file_name == Some("default.nix")
        && parent_name.is_some()
        && grandparent_name == Some("hosts")
    {
        100
    } else if file_name == Some("configuration.nix") {
        80
    } else if file_name == Some("default.nix") {
        60
    } else {
        10
    }
}

fn find_import_list(content: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut index = 0usize;
    let mut in_comment = false;
    let mut in_string = false;
    let mut in_indented_string = false;
    let mut escaped = false;
    let mut found = None;

    while index < bytes.len() {
        if in_comment {
            if bytes[index] == b'\n' {
                in_comment = false;
            }
            index += 1;
            continue;
        }

        if in_indented_string {
            if bytes[index..].starts_with(b"''") {
                in_indented_string = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if bytes[index] == b'\\' {
                escaped = true;
            } else if bytes[index] == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if bytes[index] == b'#' {
            in_comment = true;
            index += 1;
            continue;
        }

        if bytes[index] == b'"' {
            in_string = true;
            index += 1;
            continue;
        }

        if bytes[index..].starts_with(b"''") {
            in_indented_string = true;
            index += 2;
            continue;
        }

        if !bytes[index..].starts_with(b"imports") {
            index += 1;
            continue;
        }

        let before_ok = index == 0
            || !matches!(
                bytes[index - 1],
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'
            );
        let after_name = index + "imports".len();
        let after_ok = after_name == bytes.len()
            || !matches!(
                bytes[after_name],
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'
            );

        if !before_ok || !after_ok {
            index += "imports".len();
            continue;
        }

        let mut cursor = after_name;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            index += "imports".len();
            continue;
        }

        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= bytes.len() || bytes[cursor] != b'[' {
            index += "imports".len();
            continue;
        }

        if find_matching_delimiter(content, cursor, '[', ']').is_none() {
            index += "imports".len();
            continue;
        }

        if found.replace(cursor).is_some() {
            return None;
        }

        index = cursor + 1;
    }

    found
}

fn add_import_to_content(content: &str, import_path: &str) -> Result<String, String> {
    let list_start =
        find_import_list(content).ok_or_else(|| "imports list not found".to_string())?;
    let insert_at = list_start + 1;

    let indentation = content[list_start..]
        .lines()
        .nth(1)
        .map(|line| {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "  ".to_string());

    let entry = format!(
        "
{}{}",
        indentation, import_path
    );

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
    use crate::nixos::config::{ConfigFile, WriteSafety};
    use std::process::Output;

    fn test_directory(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), unique))
    }

    fn git_output(directory: &Path, args: &[&str]) -> Output {
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(args)
            .output()
            .unwrap()
    }

    #[test]
    fn rolls_back_earlier_files_when_a_later_mutation_fails() {
        let directory = test_directory("nxc-rollback-multi-file");
        fs::create_dir_all(&directory).unwrap();
        let first = directory.join("first.nix");
        let second = directory.join("second.nix");
        fs::write(&first, "first original\n").unwrap();
        fs::write(&second, "second original\n").unwrap();

        let paths = vec![first.clone(), second.clone()];
        let result = apply_file_changes(
            &paths,
            |_| Ok(()),
            |path| {
                if path == first.as_path() {
                    fs::write(path, "first modified\n")
                        .map_err(|error| error.to_string())
                } else {
                    Err("simulated failure on second file".to_string())
                }
            },
        );

        assert!(
            result.unwrap_err().contains("simulated failure on second file")
        );
        assert_eq!(fs::read_to_string(&first).unwrap(), "first original\n");
        assert_eq!(fs::read_to_string(&second).unwrap(), "second original\n");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn validates_all_targets_before_mutating_any_file() {
        let directory = test_directory("nxc-rollback-prevalidate");
        fs::create_dir_all(&directory).unwrap();
        let first = directory.join("first.nix");
        let second = directory.join("second.nix");
        fs::write(&first, "first original\n").unwrap();
        fs::write(&second, "second original\n").unwrap();

        let paths = vec![first.clone(), second.clone()];
        let result = apply_file_changes(
            &paths,
            |path| {
                if path == second.as_path() {
                    Err("simulated unsafe target".to_string())
                } else {
                    Ok(())
                }
            },
            |_| panic!("mutations must not start until all targets validate"),
        );

        assert!(result.unwrap_err().contains("simulated unsafe target"));
        assert_eq!(fs::read_to_string(&first).unwrap(), "first original\n");
        assert_eq!(fs::read_to_string(&second).unwrap(), "second original\n");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn rollback_continues_after_git_index_restore_fails() {
        let directory = test_directory("nxc-rollback-continue");
        fs::create_dir_all(&directory).unwrap();
        let original = directory.join("existing.nix");
        let created = directory.join("created.nix");
        fs::write(&original, "original\n").unwrap();
        fs::write(&created, "created by nxc\n").unwrap();
        let backup = FileBackup::capture(&original).unwrap();
        fs::write(&original, "modified\n").unwrap();

        // This path is not inside a Git repository, so index restoration fails.
        let staged = StagedFile {
            path: directory.join("not-a-git-repo.nix"),
            previous_index: None,
        };
        let result = rollback_changes(&[staged], &[backup], &[created.clone()]);

        assert!(
            result.unwrap_err().contains("failed to restore Git index")
        );
        assert_eq!(fs::read_to_string(&original).unwrap(), "original\n");
        assert!(!created.exists());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn restoring_a_staged_file_preserves_the_previous_index_entry() {
        let directory = test_directory("nxc-rollback-index");
        fs::create_dir_all(&directory).unwrap();
        let init = git_output(&directory, &["init", "--quiet"]);
        assert!(
            init.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&init.stderr)
        );

        let path = directory.join("state.nix");
        fs::write(&path, "previous staged content\n").unwrap();
        let add = git_output(&directory, &["add", "--", "state.nix"]);
        assert!(
            add.status.success(),
            "initial git add failed: {}",
            String::from_utf8_lossy(&add.stderr)
        );

        fs::write(&path, "new worktree content\n").unwrap();
        let staged = stage_new_file(&directory, &path).unwrap().unwrap();
        restore_index_state(&staged).unwrap();

        let indexed = git_output(&directory, &["show", ":state.nix"]);
        assert!(
            indexed.status.success(),
            "git show failed: {}",
            String::from_utf8_lossy(&indexed.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&indexed.stdout), "previous staged content\n");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn stage_failure_does_not_modify_the_existing_index() {
        let directory = test_directory("nxc-rollback-index-lock");
        fs::create_dir_all(&directory).unwrap();
        let init = git_output(&directory, &["init", "--quiet"]);
        assert!(init.status.success(), "git init failed: {}", String::from_utf8_lossy(&init.stderr));

        let path = directory.join("state.nix");
        fs::write(&path, "previous staged content\n").unwrap();
        let add = git_output(&directory, &["add", "--", "state.nix"]);
        assert!(add.status.success(), "initial git add failed: {}", String::from_utf8_lossy(&add.stderr));
        fs::write(&path, "new worktree content\n").unwrap();

        fs::write(directory.join(".git/index.lock"), "locked").unwrap();
        let error = stage_new_file(&directory, &path).unwrap_err();
        fs::remove_file(directory.join(".git/index.lock")).unwrap();

        assert!(error.contains("failed to stage newly created NXC file"));
        assert!(!error.contains("rollback also failed"));
        let indexed = git_output(&directory, &["show", ":state.nix"]);
        assert!(indexed.status.success(), "git show failed: {}", String::from_utf8_lossy(&indexed.stderr));
        assert_eq!(String::from_utf8_lossy(&indexed.stdout), "previous staged content\n");
        let _ = fs::remove_dir_all(directory);
    }



    #[cfg(unix)]
    #[test]
    fn preserves_dangling_package_module_symlink_on_create_failure() {
        use std::os::unix::fs::symlink;

        let directory = test_directory("nxc-dangling-package-module");
        fs::create_dir_all(directory.join("nxc")).unwrap();
        let config_path = directory.join("configuration.nix");
        fs::write(&config_path, "{ imports = [ ]; }\n").unwrap();

        let module_path = dedicated_package_path(&directory);
        let missing_target = directory.join("packages-target.nix");
        symlink(&missing_target, &module_path).unwrap();
        let config = ConfigState {
            files: vec![config_file(config_path.to_str().unwrap(), &[])],
        };

        let error = install_package(&directory, &config, "firefox").unwrap_err();

        assert!(error.contains("failed to create"));
        assert_eq!(fs::read_link(&module_path).unwrap(), missing_target);
        assert!(!missing_target.exists());
        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn preserves_dangling_service_module_symlink_on_create_failure() {
        use std::os::unix::fs::symlink;

        let directory = test_directory("nxc-dangling-service-module");
        fs::create_dir_all(directory.join("nxc")).unwrap();
        let config_path = directory.join("configuration.nix");
        fs::write(&config_path, "{ imports = [ ]; }\n").unwrap();

        let module_path = dedicated_service_path(&directory);
        let missing_target = directory.join("services-target.nix");
        symlink(&missing_target, &module_path).unwrap();
        let config = ConfigState {
            files: vec![config_file(config_path.to_str().unwrap(), &[])],
        };

        let error = enable_service(&directory, &config, "openssh").unwrap_err();

        assert!(error.contains("failed to create"));
        assert_eq!(fs::read_link(&module_path).unwrap(), missing_target);
        assert!(!missing_target.exists());
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn removes_new_module_when_import_update_fails() {
        let directory = test_directory("nxc-rollback-new-module-import");
        fs::create_dir_all(&directory).unwrap();
        let config_path = directory.join("configuration.nix");
        fs::write(&config_path, "{ }\n").unwrap();

        let config = ConfigState {
            files: vec![config_file(config_path.to_str().unwrap(), &[])],
        };
        let result = install_package(&directory, &config, "firefox");

        assert!(result.unwrap_err().contains("imports list not found"));
        assert!(!dedicated_package_path(&directory).exists());
        assert_eq!(fs::read_to_string(&config_path).unwrap(), "{ }\n");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn restores_import_and_module_when_package_declaration_fails() {
        let directory = test_directory("nxc-rollback-import-declaration");
        fs::create_dir_all(&directory).unwrap();
        let config_path = directory.join("configuration.nix");
        fs::write(&config_path, "{ imports = [ ]; }\n").unwrap();

        let module_path = dedicated_package_path(&directory);
        fs::create_dir_all(module_path.parent().unwrap()).unwrap();
        let original_module = "{ pkgs, ... }:\n\n{\n  environment.systemPackages = with pkgs; [\n    firefox\n  ];\n}\n";
        fs::write(&module_path, original_module).unwrap();
        let original_config = fs::read_to_string(&config_path).unwrap();
        let config = ConfigState {
            files: vec![config_file(config_path.to_str().unwrap(), &[])],
        };

        let result = install_package(&directory, &config, "firefox");

        assert!(result.unwrap_err().contains("already present"));
        assert_eq!(fs::read_to_string(&module_path).unwrap(), original_module);
        assert_eq!(fs::read_to_string(&config_path).unwrap(), original_config);
        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn restoring_a_backup_preserves_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = test_directory("nxc-rollback-permissions");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.nix");
        fs::write(&path, "original\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        let backup = FileBackup::capture(&path).unwrap();
        fs::write(&path, "modified\n").unwrap();
        backup.restore().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "original\n");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn rejects_existing_package_file_without_nxc_shape() {
        let dir = std::env::temp_dir().join(format!("nxc-dedicated-pkg-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("packages.nix");
        fs::write(&path, "{ environment.systemPackages = [ pkgs.git ]; }").unwrap();

        assert!(validate_dedicated_package_module(&path).is_err());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn accepts_existing_service_file_with_nxc_shape() {
        let dir = std::env::temp_dir().join(format!("nxc-dedicated-svc-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("services.nix");
        fs::write(&path, "{ ... }:\n\n{\n}\n").unwrap();

        assert!(validate_dedicated_service_module(&path).is_ok());

        let _ = fs::remove_dir_all(dir);
    }

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

    #[test]
    fn rejects_multiple_import_lists() {
        let content = r#"
{
  imports = [ ./hardware.nix ];
  imports = [ ./desktop.nix ];
}
"#;

        assert!(find_import_list(content).is_none());
    }

    fn config_file(path: &str, imports: &[&str]) -> ConfigFile {
        ConfigFile {
            path: PathBuf::from(path),
            has_system_packages: false,
            has_systemd_services: false,
            has_service_options: false,
            declared_packages: Default::default(),
            declared_services: Default::default(),
            imports: imports.iter().map(PathBuf::from).collect(),
            write_safety: WriteSafety::Unsafe,
        }
    }

    #[test]
    fn selects_host_default_as_import_graph_root() {
        let root = PathBuf::from("/tmp/nix-config");
        let config = ConfigState {
            files: vec![
                config_file(
                    "/tmp/nix-config/hosts/laptop/default.nix",
                    &[
                        "/tmp/nix-config/hosts/laptop/hardware-configuration.nix",
                        "/tmp/nix-config/hosts/common.nix",
                    ],
                ),
                config_file(
                    "/tmp/nix-config/hosts/laptop/hardware-configuration.nix",
                    &[],
                ),
                config_file("/tmp/nix-config/hosts/common.nix", &[]),
                config_file("/tmp/nix-config/modules/core.nix", &[]),
            ],
        };

        assert_eq!(
            find_import_target(&config).unwrap(),
            PathBuf::from("/tmp/nix-config/hosts/laptop/default.nix")
        );
    }

    #[test]
    fn prefers_configuration_root_when_no_host_default_exists() {
        let root = PathBuf::from("/tmp/nix-config");
        let config = ConfigState {
            files: vec![
                config_file("/tmp/nix-config/configuration.nix", &[]),
                config_file("/tmp/nix-config/hardware.nix", &[]),
            ],
        };

        assert_eq!(
            find_import_target(&config).unwrap(),
            PathBuf::from("/tmp/nix-config/configuration.nix")
        );
    }

    #[test]
    fn accepts_single_non_conventional_graph_root() {
        let config = ConfigState {
            files: vec![
                config_file(
                    "/tmp/nix-config/system.nix",
                    &["/tmp/nix-config/modules/core.nix"],
                ),
                config_file("/tmp/nix-config/modules/core.nix", &[]),
            ],
        };

        assert_eq!(
            find_import_target(&config).unwrap(),
            PathBuf::from("/tmp/nix-config/system.nix")
        );
    }

    #[test]
    fn rejects_ambiguous_graph_roots_with_equal_priority() {
        let root = PathBuf::from("/tmp/nix-config");
        let config = ConfigState {
            files: vec![
                config_file("/tmp/nix-config/hosts/laptop/default.nix", &[]),
                config_file("/tmp/nix-config/hosts/desktop/default.nix", &[]),
            ],
        };

        let error = find_import_target(&config).unwrap_err();

        assert!(error.contains("multiple NixOS module graph roots"));
    }

    #[test]
    fn ignores_comment_and_string_mentions() {
        let content = r#"
{
  # imports = [ ./ignored.nix ];
  description = "imports = [ ./ignored.nix ]";
}
"#;

        assert!(find_import_list(content).is_none());
    }
}
