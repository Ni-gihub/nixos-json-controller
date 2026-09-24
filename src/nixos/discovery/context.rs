use std::fs;
use std::path::{Path, PathBuf};

use super::{
    load_discovery,
    DiscoveryResult,
    FileMetadata,
};

/// 通常のnxc操作で利用するDiscoveryコンテキスト。
///
/// Discoveryそのものは実行せず、保存済みの結果だけを利用する。
#[derive(Debug, Clone)]
pub struct DiscoveryContext {
    result: DiscoveryResult,
}

impl DiscoveryContext {
    /// 保存済みDiscoveryResultからContextを作る。
    ///
    /// ここでは重いNix評価を行わない。
    /// 通常操作を軽く保つため、ファイルシステム上の
    /// 軽量なstale判定だけを行う。
    pub fn load() -> Result<Self, String> {
        let result = load_discovery()
            .map_err(|_| {
                Self::discovery_required_message()
            })?;

        Self::validate_result(&result)?;

        Ok(Self { result })
    }

    /// DiscoveryResultそのものを取得する。
    pub fn result(&self) -> &DiscoveryResult {
        &self.result
    }

    /// Flake root
    pub fn flake_root(&self) -> &Path {
        &self.result.flake_root
    }

    /// flake.nix
    pub fn flake_file(&self) -> &Path {
        &self.result.flake_file
    }

    /// 選択されたconfiguration
    pub fn configuration_name(&self) -> &str {
        &self
            .result
            .selected_configuration
            .name
    }

    /// `flake-root#configuration`
    pub fn flake_reference(&self) -> String {
        format!(
            "{}#{}",
            self.flake_root().display(),
            self.configuration_name()
        )
    }

    fn validate_result(
        result: &DiscoveryResult,
    ) -> Result<(), String> {
        Self::validate_paths(result)?;
        Self::validate_configuration(result)?;
        Self::validate_environment(result)?;
        Self::validate_metadata(result)?;

        Ok(())
    }

    fn validate_paths(
        result: &DiscoveryResult,
    ) -> Result<(), String> {
        if result
            .flake_root
            .as_os_str()
            .is_empty()
        {
            return Err(
                Self::rediscover_message(
                    "discovered flake root is empty",
                )
            );
        }

        if !result.flake_root.is_dir() {
            return Err(
                Self::rediscover_message(
                    &format!(
                        "discovered flake directory does not exist: {}",
                        result.flake_root.display()
                    ),
                )
            );
        }

        if !result.flake_file.is_file() {
            return Err(
                Self::rediscover_message(
                    &format!(
                        "discovered flake.nix does not exist: {}",
                        result.flake_file.display()
                    ),
                )
            );
        }

        let expected_flake_file =
            result.flake_root.join("flake.nix");

        if result.flake_file != expected_flake_file {
            return Err(
                Self::rediscover_message(
                    "discovered flake.nix path does not belong to the discovered flake root",
                )
            );
        }

        Ok(())
    }

    fn validate_configuration(
        result: &DiscoveryResult,
    ) -> Result<(), String> {
        let name = result
            .selected_configuration
            .name
            .trim();

        if name.is_empty() {
            return Err(
                Self::rediscover_message(
                    "discovered NixOS configuration name is empty",
                )
            );
        }

        if name.contains('/') {
            return Err(
                Self::rediscover_message(
                    "discovered NixOS configuration name is invalid",
                )
            );
        }

        Ok(())
    }

    /// 現在のマシンとDiscovery時点の環境を比較する。
    ///
    /// ここでもNix評価は行わない。
    fn validate_environment(
        result: &DiscoveryResult,
    ) -> Result<(), String> {
        let configuration =
            &result.selected_configuration;

        if let Some(saved_hostname) =
            configuration.hostname.as_deref()
        {
            let current_hostname =
                current_hostname();

            if let Some(current_hostname) =
                current_hostname
            {
                if current_hostname != saved_hostname {
                    return Err(
                        Self::rediscover_message(
                            &format!(
                                "current hostname '{}' does not match discovered hostname '{}'",
                                current_hostname,
                                saved_hostname
                            ),
                        )
                    );
                }
            }
        }

        if let Some(saved_system) =
            configuration.system.as_deref()
        {
            let current_system =
                current_system();

            if current_system != saved_system {
                return Err(
                    Self::rediscover_message(
                        &format!(
                            "current system '{}' does not match discovered system '{}'",
                            current_system,
                            saved_system
                        ),
                    )
                );
            }
        }

        Ok(())
    }

    /// Discovery時点からflakeが変更されていないか確認する。
    ///
    /// flake.nix / flake.lock の変更を検出したら、
    /// 通常操作では勝手に再探索せず停止する。
    fn validate_metadata(
        result: &DiscoveryResult,
    ) -> Result<(), String> {
        let metadata =
            result.metadata.as_ref().ok_or_else(|| {
                Self::rediscover_message(
                    "discovery state does not contain stale-check metadata",
                )
            })?;

        let current_flake_nix =
            FileMetadata::from_path(
                &result.flake_file,
            )?;

        if current_flake_nix != metadata.flake_nix {
            return Err(
                Self::rediscover_message(
                    "flake.nix has changed since discovery",
                )
            );
        }

        let flake_lock =
            result.flake_root.join("flake.lock");

        match (
            metadata.flake_lock.as_ref(),
            flake_lock.is_file(),
        ) {
            (Some(saved), true) => {
                let current =
                    FileMetadata::from_path(
                        &flake_lock,
                    )?;

                if &current != saved {
                    return Err(
                        Self::rediscover_message(
                            "flake.lock has changed since discovery",
                        )
                    );
                }
            }

            (Some(_), false) => {
                return Err(
                    Self::rediscover_message(
                        "flake.lock was removed since discovery",
                    )
                );
            }

            (None, true) => {
                return Err(
                    Self::rediscover_message(
                        "flake.lock was created since discovery",
                    )
                );
            }

            (None, false) => {}
        }

        Ok(())
    }

    fn discovery_required_message() -> String {
        "NixOS flake discovery is not available. Run `nxc discover` first."
            .to_string()
    }

    fn rediscover_message(
        reason: &str,
    ) -> String {
        format!(
            "{}. Run `nxc discover` again.",
            reason
        )
    }

    /// Contextからflake rootをPathBufとして取得する。
    pub fn flake_root_buf(&self) -> PathBuf {
        self.flake_root().to_path_buf()
    }
}

/// 現在のhostnameを取得する。
///
/// HOSTNAMEが無い環境では /etc/hostname を利用する。
fn current_hostname() -> Option<String> {
    if let Some(hostname) =
        std::env::var_os("HOSTNAME")
    {
        let hostname =
            hostname
                .to_string_lossy()
                .trim()
                .to_string();

        if !hostname.is_empty() {
            return Some(hostname);
        }
    }

    let hostname =
        fs::read_to_string(
            "/etc/hostname",
        )
        .ok()?;

    let hostname =
        hostname
            .trim()
            .to_string();

    if hostname.is_empty() {
        None
    } else {
        Some(hostname)
    }
}

/// 現在のRust実行環境からNix system形式を作る。
///
/// 例:
/// `x86_64-linux`
fn current_system() -> String {
    format!(
        "{}-{}",
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}