use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::module;

pub fn install_package(path: &Path, package: &str) -> Result<(), String> {
    let content = read(path)?;
    let updated = module::add_package_to_content(&content, package)?;
    write_atomic(path, &updated)
}

pub fn remove_package(path: &Path, package: &str) -> Result<(), String> {
    let content = read(path)?;
    let updated = module::remove_package_from_content(&content, package)?;
    write_atomic(path, &updated)
}

pub fn enable_service(path: &Path, service: &str) -> Result<(), String> {
    let content = read(path)?;
    let updated = module::add_service_to_content(&content, service, true)?;
    write_atomic(path, &updated)
}

pub fn disable_service(path: &Path, service: &str) -> Result<(), String> {
    let content = read(path)?;
    let updated = module::add_service_to_content(&content, service, false)?;
    write_atomic(path, &updated)
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid output path: {}", path.display()))?;

    let metadata = fs::metadata(path)
        .map_err(|e| format!("failed to inspect {}: {}", path.display(), e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("failed to create temporary path: {}", e))?
        .as_nanos();

    let temporary = parent.join(format!(".{}.nxc-{}", file_name, timestamp));

    fs::write(&temporary, content)
        .map_err(|e| format!("failed to write temporary file {}: {}", temporary.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, metadata.permissions())
            .map_err(|e| format!("failed to preserve permissions on {}: {}", temporary.display(), e))?;
    }

    let result = fs::rename(&temporary, path);
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!("failed to replace {}: {}", path.display(), error));
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

        install_package(&path, "firefox").unwrap();

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

        remove_package(&path, "firefox").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("\nfirefox\n"));
        assert!(content.contains("\ngit\n"));

        let _ = fs::remove_dir_all(directory);
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

        enable_service(&path, "openssh").unwrap();

        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);

        let _ = fs::remove_dir_all(directory);
    }
}
