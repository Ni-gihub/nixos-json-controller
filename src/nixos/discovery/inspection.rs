use serde::{Deserialize, Serialize};

/// ファイルシステムの検査結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemInspection {
    pub flake_file_exists: bool,
    pub flake_lock_exists: bool,
    pub is_git_repository: bool,

    pub has_hosts_directory: bool,
    pub has_modules_directory: bool,
    pub has_home_directory: bool,
    pub has_system_directory: bool,

    pub has_configuration_nix: bool,
    pub has_hardware_configuration_nix: bool,

    /// flake.nix内にnixosConfigurationsという文字列があるか
    ///
    /// これは正式な判定には使用しない。
    /// 軽量な手掛かりとしてのみ利用する。
    pub contains_nixos_configurations_text: bool,
}

/// Nix評価時のエラー分類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvaluationErrorCategory {
    CommandNotFound,
    InvalidFlake,
    EvaluationFailed,
    PermissionDenied,
    Timeout,
    Unknown,
}

/// Nix評価時のエラー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixEvaluationError {
    pub category: EvaluationErrorCategory,
    pub message: String,
}

/// NixOS Configurationの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixosConfiguration {
    /// nixosConfigurations内の属性名
    pub name: String,

    /// systemの種類
    /// 例: x86_64-linux
    pub system: Option<String>,

    /// Configurationから取得できたhostname
    pub hostname: Option<String>,

    /// プラットフォーム情報
    pub platform: Option<String>,
}

/// Flakeのoutputs情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlakeOutputs {
    pub nixos_configurations: Vec<NixosConfiguration>,
}

/// Nixによる評価結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NixEvaluation {
    NotEvaluated,

    Success {
        outputs: FlakeOutputs,
    },

    Failed {
        error: NixEvaluationError,
    },
}

/// 現在の環境との照合結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentMatch {
    pub current_hostname: Option<String>,
    pub current_system: Option<String>,

    pub hostname_matches: Vec<String>,
    pub system_matches: Vec<String>,
}

/// 候補の検査結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub filesystem: FilesystemInspection,
    pub nix_evaluation: NixEvaluation,
    pub environment_match: EnvironmentMatch,
}