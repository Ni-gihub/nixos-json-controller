use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::module;

pub fn install_package(flake_root: &Path, path: &Path, package: &str) -> Result<(), String> {
    let content = read(flake_root, path)?;
    let updated = module::add_package_to_content(&content, package)?;
    write_atomic(flake_root, path, &updated)
}

pub fn remove_package(flake_root: &Path, path: &Path, package: &str) -> Result<(), String> {
    let content = read(flake_root, path)?;
    let updated = module::remove_package_from_content(&content, package)?;
    write_atomic(flake_root, path, &updated)
}

pub fn enable_service(flake_root: &Path, path: &Path, service: &str) -> Result<(), String> {
    let content = read(flake_root, path)?;
    let updated = module::add_service_to_content(&content, service, true)?;
    write_atomic(flake_root, path, &updated)
}

pub fn disable_service(flake_root: &Path, path: &Path, service: &str) -> Result<(), String> {
    let content = read(flake_root, path)?;
    let updated = module::add_service_to_content(&content, service, false)?;
    write_atomic(flake_root, path, &updated)
}

fn read(flake_root: &Path, path: &Path) -> Result<String, String> {
    let target = resolve_write_target(flake_root, path)?;
    fs::read_to_string(&target)
        .map_err(|e| format!("failed to read {}: {}", target.display(), e))
}

/// Resolve a path and enforce the same flake-root boundary used by discovery.
/// Both the read and the later atomic replacement call this check so a symlink
/// retargeted after discovery cannot redirect a write outside the flake.
pub(crate) fn resolve_write_target(flake_root: &Path, path: &Path) -> Result<std::path::PathBuf, String> {
    let root = fs::canonicalize(flake_root).map_err(|error| {
        format!(
            "failed to resolve flake root {}: {}",
            flake_root.display(),
            error
        )
    })?;
    let target = fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to resolve write target {} (broken or cyclic symlink?): {}",
            path.display(),
            error
        )
    })?;

    if !target.starts_with(&root) {
        return Err(format!(
            "refusing to write {}: resolved target {} is outside flake root {}",
            path.display(),
            target.display(),
            root.display()
        ));
    }

    let metadata = fs::metadata(&target)
        .map_err(|error| format!("failed to inspect {}: {}", target.display(), error))?;
    if !metadata.is_file() {
        return Err(format!(
            "refusing to write {}: resolved target {} is not a regular file",
            path.display(),
            target.display()
        ));
    }

    Ok(target)
}

pub(crate) fn write_atomic(
    flake_root: &Path,
    path: &Path,
    content: &str,
) -> Result<(), String> {
    // Resolve the destination before creating and renaming the temporary file.
    // Renaming over the lexical symlink path would replace the symlink itself;
    // renaming over the canonical target updates its contents and preserves the
    // link for both normal writes and rollback.
    let destination = resolve_write_target(flake_root, path)?;
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid output path: {}", destination.display()))?;

    let metadata = fs::metadata(&destination)
        .map_err(|e| format!("failed to inspect {}: {}", destination.display(), e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("failed to create temporary path: {}", e))?
        .as_nanos();

    let temporary = parent.join(format!(".{}.nxc-{}", file_name, timestamp));

    if let Err(error) = fs::write(&temporary, content) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to write temporary file {}: {}",
            temporary.display(),
            error
        ));
    }

    #[cfg(unix)]
    {
        if let Err(error) = fs::set_permissions(&temporary, metadata.permissions()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "failed to preserve permissions on {}: {}",
                temporary.display(),
                error
            ));
        }
    }

    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to replace {} (resolved from {}): {}",
            destination.display(),
            path.display(),
            error
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_package_changes_atomically() {
        let directory = std::env::temp_dir().join(format!("nxc-writer-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();

        let path = directory.join("packages.nix");
        fs::write(&path, "environment.systemPackages = with pkgs; [\n];\n").unwrap();

        install_package(&directory, &path, "firefox").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("firefox"));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn removes_package_from_file() {
        let directory =
            std::env::temp_dir().join(format!("nxc-writer-remove-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();

        let path = directory.join("packages.nix");
        fs::write(
            &path,
            "environment.systemPackages = with pkgs; [\nfirefox\ngit\n];\n",
        )
        .unwrap();

        remove_package(&directory, &path, "firefox").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("\nfirefox\n"));
        assert!(content.contains("\ngit\n"));

        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn atomically_updates_symlink_target_without_replacing_the_link() {
        use std::os::unix::fs::symlink;

        let directory = std::env::temp_dir().join(format!(
            "nxc-writer-symlink-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(directory.join("real")).unwrap();

        let target = directory.join("real/packages.nix");
        let link = directory.join("packages.nix");
        fs::write(&target, "environment.systemPackages = with pkgs; [\n];\n").unwrap();
        symlink(&target, &link).unwrap();

        install_package(&directory, &link, "firefox").unwrap();

        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(&link).unwrap(), target);
        assert!(fs::read_to_string(&target).unwrap().contains("firefox"));

        let _ = fs::remove_dir_all(directory);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_write_through_symlink_outside_flake_root() {
        use std::os::unix::fs::symlink;

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let parent = std::env::temp_dir().join(format!(
            "nxc-writer-outside-symlink-{}-{}",
            std::process::id(),
            unique
        ));
        let root = parent.join("flake");
        let outside = parent.join("outside.nix");
        fs::create_dir_all(&root).unwrap();
        let original = "environment.systemPackages = with pkgs; [\n];\n";
        fs::write(&outside, original).unwrap();
        let link = root.join("packages.nix");
        symlink(&outside, &link).unwrap();

        let error = install_package(&root, &link, "firefox").unwrap_err();

        assert!(error.contains("outside flake root"));
        assert_eq!(fs::read_to_string(&outside).unwrap(), original);
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(&link).unwrap(), outside);

        let _ = fs::remove_dir_all(parent);
    }

    #[cfg(unix)]
    #[test]
    fn preserves_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory =
            std::env::temp_dir().join(format!("nxc-writer-perms-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();

        let path = directory.join("config.nix");
        fs::write(&path, "{\n}\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        enable_service(&directory, &path, "openssh").unwrap();

        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );

        let _ = fs::remove_dir_all(directory);
    }
}
