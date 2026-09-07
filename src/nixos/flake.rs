use std::{
    env, fs,
    path::{Path, PathBuf},
};

const NXC_FLAKE_ENV: &str = "NXC_FLAKE";
const DEFAULT_FLAKE_DIR: &str = "Projects/nix-config";

/// NixOS configuration flake を探す。
///
/// 探索順:
/// 1. NXC_FLAKE が設定されていれば、それを使用
/// 2. 現在のディレクトリから親ディレクトリへ遡って探索
/// 3. HOME/Projects/nix-config を探索
pub fn repository_path() -> Result<PathBuf, String> {
    // ------------------------------------------------------------
    // 1. 環境変数 NXC_FLAKE
    // ------------------------------------------------------------
    if let Ok(path) = env::var(NXC_FLAKE_ENV) {
        let path = PathBuf::from(path);

        if is_nixos_flake(&path) {
            return Ok(path);
        }

        return Err(format!("NixOS flake not found: {}", path.display()));
    }

    // ------------------------------------------------------------
    // 2. 現在のディレクトリから親へ遡って探索
    // ------------------------------------------------------------
    if let Ok(current_dir) = env::current_dir() {
        for ancestor in current_dir.ancestors() {
            if is_nixos_flake(ancestor) {
                return Ok(ancestor.to_path_buf());
            }
        }
    }

    // ------------------------------------------------------------
    // 3. デフォルトパス
    // ------------------------------------------------------------
    if let Ok(home) = env::var("HOME") {
        let default_path = PathBuf::from(home).join(DEFAULT_FLAKE_DIR);

        if is_nixos_flake(&default_path) {
            return Ok(default_path);
        }
    }

    Err("NixOS flake not found".to_string())
}

/// 指定されたディレクトリが NixOS configuration flake か判定する。
fn is_nixos_flake(path: &Path) -> bool {
    let flake_path = path.join("flake.nix");

    if !flake_path.is_file() {
        return false;
    }

    let flake = match fs::read_to_string(&flake_path) {
        Ok(content) => content,
        Err(_) => return false,
    };

    // NixOS configuration を持つ flake であることを確認する。
    flake.contains("nixosConfigurations")
}

/// packages.nix のパス。
pub fn pkgs_path() -> Result<PathBuf, String> {
    Ok(repository_path()?.join("modules").join("pkgs.nix"))
}

/// core.nix のパス。
fn core_path() -> Result<PathBuf, String> {
    Ok(repository_path()?.join("modules").join("core.nix"))
}

/// pkgs.nix を書き込む。
pub fn write_pkgs(content: &str) -> Result<(), String> {
    fs::write(pkgs_path()?, content).map_err(|e| e.to_string())
}

/// core.nix を書き込む。
pub fn write_core(content: &str) -> Result<(), String> {
    fs::write(core_path()?, content).map_err(|e| e.to_string())
}

/// pkgs.nix を読む。
pub fn read_pkgs() -> Result<String, String> {
    fs::read_to_string(pkgs_path()?).map_err(|e| e.to_string())
}

/// core.nix を読む。
pub fn read_core() -> Result<String, String> {
    fs::read_to_string(core_path()?).map_err(|e| e.to_string())
}
