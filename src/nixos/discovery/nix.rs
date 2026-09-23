use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    EnvironmentMatch,
    EvaluationErrorCategory,
    FilesystemInspection,
    FlakeCandidate,
    FlakeOutputs,
    InspectionResult,
    NixEvaluation,
    NixEvaluationError,
    NixosConfiguration,
};

/// `nix flake metadata --json` の実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixFlakeMetadata {
    pub description: Option<String>,
    pub path: Option<String>,

    #[serde(rename = "lastModified")]
    pub last_modified: Option<u64>,

    #[serde(rename = "lastModifiedDate")]
    pub last_modified_date: Option<String>,

    pub locked: Option<Value>,
}

/// `nix flake show --json` の実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixFlakeShow {
    #[serde(rename = "nixosConfigurations", default)]
    pub nixos_configurations: HashMap<String, NixFlakeOutput>,
}

/// Flake output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixFlakeOutput {
    #[serde(rename = "type")]
    pub output_type: Option<String>,
}

/// Nix評価時のエラー
#[derive(Debug, Clone)]
pub struct NixCommandError {
    pub message: String,
}

impl std::fmt::Display for NixCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NixCommandError {}

/// `nix flake metadata --json` を評価する
pub fn evaluate_flake(
    flake_root: &Path,
) -> Result<NixFlakeMetadata, NixCommandError> {
    let output = Command::new("nix")
        .args([
            "flake",
            "metadata",
            "--json",
        ])
        .current_dir(flake_root)
        .output()
        .map_err(|error| NixCommandError {
            message: format!("failed to execute nix: {error}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(NixCommandError {
            message: format!(
                "nix flake metadata failed for {}: {}",
                flake_root.display(),
                stderr.trim()
            ),
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|error| {
        NixCommandError {
            message: format!(
                "failed to parse nix flake metadata output: {error}"
            ),
        }
    })
}

/// `nix flake show --json` を評価する
pub fn show_flake(
    flake_root: &Path,
) -> Result<NixFlakeShow, NixCommandError> {
    let output = Command::new("nix")
        .args([
            "flake",
            "show",
            "--json",
        ])
        .current_dir(flake_root)
        .output()
        .map_err(|error| NixCommandError {
            message: format!("failed to execute nix: {error}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(NixCommandError {
            message: format!(
                "nix flake show failed for {}: {}",
                flake_root.display(),
                stderr.trim()
            ),
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|error| {
        NixCommandError {
            message: format!(
                "failed to parse nix flake show output: {error}"
            ),
        }
    })
}

/// NixOS Configurationを評価する
pub fn evaluate_nixos_configuration(
    flake_root: &Path,
    configuration_name: &str,
) -> Result<NixosConfiguration, NixCommandError> {
    let system = evaluate_attribute(
        flake_root,
        configuration_name,
        "config.system.build.toplevel.system",
    )?;

    let hostname = evaluate_attribute(
        flake_root,
        configuration_name,
        "config.networking.hostName",
    )?;

    Ok(NixosConfiguration {
        name: configuration_name.to_string(),
        system: Some(system),
        hostname: Some(hostname),
        platform: None,
    })
}

/// Nix attributeをJSONとして評価する
fn evaluate_attribute(
    flake_root: &Path,
    configuration_name: &str,
    attribute: &str,
) -> Result<String, NixCommandError> {
    let attribute_path = format!(
        ".#nixosConfigurations.{configuration_name}.{attribute}"
    );

    let output = Command::new("nix")
        .args([
            "eval",
            "--json",
            &attribute_path,
        ])
        .current_dir(flake_root)
        .output()
        .map_err(|error| NixCommandError {
            message: format!("failed to execute nix: {error}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(NixCommandError {
            message: format!(
                "nix eval failed for {}: {}",
                attribute_path,
                stderr.trim()
            ),
        });
    }

    serde_json::from_slice::<String>(&output.stdout).map_err(|error| {
        NixCommandError {
            message: format!(
                "failed to parse nix eval output for {}: {error}",
                attribute_path
            ),
        }
    })
}

/// Flake候補をNixまで含めて検査する
pub fn inspect_flake(
    candidate: &FlakeCandidate,
) -> InspectionResult {
    let filesystem = inspect_filesystem(candidate);

    let nix_evaluation = match evaluate_outputs(
        &candidate.flake_root,
    ) {
        Ok(outputs) => NixEvaluation::Success { outputs },

        Err(error) => NixEvaluation::Failed {
            error: NixEvaluationError {
                category: classify_error(&error),
                message: error.message,
            },
        },
    };

    let environment_match = match &nix_evaluation {
        NixEvaluation::Success { outputs } => {
            match_environment(outputs)
        }

        NixEvaluation::Failed { .. }
        | NixEvaluation::NotEvaluated => {
            EnvironmentMatch {
                current_hostname: current_hostname(),
                current_system: current_system(),
                hostname_matches: Vec::new(),
                system_matches: Vec::new(),
            }
        }
    };

    InspectionResult {
        filesystem,
        nix_evaluation,
        environment_match,
    }
}

/// ファイルシステム上の情報を検査する
fn inspect_filesystem(
    candidate: &FlakeCandidate,
) -> FilesystemInspection {
    let root = &candidate.flake_root;

    FilesystemInspection {
        flake_file_exists: candidate.flake_file.is_file(),

        flake_lock_exists:
            root.join("flake.lock").is_file(),

        is_git_repository:
            root.join(".git").is_dir(),

        has_hosts_directory:
            root.join("hosts").is_dir(),

        has_modules_directory:
            root.join("modules").is_dir(),

        has_home_directory:
            root.join("home").is_dir(),

        has_system_directory:
            root.join("system").is_dir(),

        has_configuration_nix:
            root.join("configuration.nix").is_file(),

        has_hardware_configuration_nix:
            root.join("hardware-configuration.nix").is_file(),

        contains_nixos_configurations_text:
            candidate
                .evidence
                .contains(
                    &super::Evidence::NixosConfigurationsText
                ),
    }
}

/// FlakeのoutputsをNixで評価する
fn evaluate_outputs(
    flake_root: &Path,
) -> Result<FlakeOutputs, NixCommandError> {
    let show = show_flake(flake_root)?;

    let mut configurations = Vec::new();

    for name in show.nixos_configurations.keys() {
        let configuration =
            evaluate_nixos_configuration(
                flake_root,
                name,
            )?;

        configurations.push(configuration);
    }

    Ok(FlakeOutputs {
        nixos_configurations: configurations,
    })
}

/// Nix評価エラーをDiscovery用カテゴリへ分類する
fn classify_error(
    error: &NixCommandError,
) -> EvaluationErrorCategory {
    let message = error.message.to_lowercase();

    if message.contains("permission denied") {
        EvaluationErrorCategory::PermissionDenied
    } else if message.contains("timed out")
        || message.contains("timeout")
    {
        EvaluationErrorCategory::Timeout
    } else if message.contains("command not found")
        || message.contains("failed to execute nix")
    {
        EvaluationErrorCategory::CommandNotFound
    } else if message.contains("flake") {
        EvaluationErrorCategory::InvalidFlake
    } else if message.contains("eval")
        || message.contains("evaluation")
    {
        EvaluationErrorCategory::EvaluationFailed
    } else {
        EvaluationErrorCategory::Unknown
    }
}

/// 現在のhostnameを取得する
fn current_hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|hostname| hostname.trim().to_string())
        .filter(|hostname| !hostname.is_empty())
}

/// 現在のNixシステムを取得する
fn current_system() -> Option<String> {
    let output = Command::new("nix")
        .args([
            "eval",
            "--impure",
            "--raw",
            "--expr",
            "builtins.currentSystem",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let system =
        String::from_utf8(output.stdout).ok()?;

    let system = system.trim();

    if system.is_empty() {
        None
    } else {
        Some(system.to_string())
    }
}

/// 現在の環境とNixOS Configurationを比較する
fn match_environment(
    outputs: &FlakeOutputs,
) -> EnvironmentMatch {
    let current_hostname = current_hostname();
    let current_system = current_system();

    let hostname_matches = outputs
        .nixos_configurations
        .iter()
        .filter(|configuration| {
            match (
                &current_hostname,
                &configuration.hostname,
            ) {
                (Some(current), Some(hostname)) => {
                    current == hostname
                }
                _ => false,
            }
        })
        .map(|configuration| {
            configuration.name.clone()
        })
        .collect();

    let system_matches = outputs
        .nixos_configurations
        .iter()
        .filter(|configuration| {
            match (
                &current_system,
                &configuration.system,
            ) {
                (Some(current), Some(system)) => {
                    current == system
                }
                _ => false,
            }
        })
        .map(|configuration| {
            configuration.name.clone()
        })
        .collect();

    EnvironmentMatch {
        current_hostname,
        current_system,
        hostname_matches,
        system_matches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NIX_CONFIG_PATH: &str =
        "/home/nakaoku/Projects/nix-config";

    #[test]
    fn evaluate_nix_config_flake() {
        let flake_root =
            Path::new(NIX_CONFIG_PATH);

        let metadata =
            evaluate_flake(flake_root)
                .expect(
                    "failed to evaluate nix-config flake"
                );

        println!("{metadata:#?}");

        assert!(metadata.path.is_some());
    }

    #[test]
    fn show_nix_config_flake() {
        let flake_root =
            Path::new(NIX_CONFIG_PATH);

        let show =
            show_flake(flake_root)
                .expect(
                    "failed to show nix-config flake"
                );

        println!("{show:#?}");

        assert!(
            show.nixos_configurations
                .contains_key("laptop")
        );

        let laptop =
            show.nixos_configurations
                .get("laptop")
                .expect(
                    "laptop configuration not found"
                );

        assert_eq!(
            laptop.output_type.as_deref(),
            Some("nixos-configuration")
        );
    }

    #[test]
    fn evaluate_laptop_configuration() {
        let flake_root =
            Path::new(NIX_CONFIG_PATH);

        let configuration =
            evaluate_nixos_configuration(
                flake_root,
                "laptop",
            )
            .expect(
                "failed to evaluate laptop configuration"
            );

        println!("{configuration:#?}");

        assert_eq!(
            configuration.name,
            "laptop"
        );

        assert_eq!(
            configuration.system.as_deref(),
            Some("x86_64-linux")
        );

        assert_eq!(
            configuration.hostname.as_deref(),
            Some("nixos-laptop")
        );
    }
}