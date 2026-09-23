use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::inspection::NixosConfiguration;
use super::candidate::FlakeCandidate;

/// Flakeの選択方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelectionMethod {
    ExplicitCli,
    ConfigFile,
    EnvironmentVariable,
    UniqueCandidate,
    HostnameMatch,
    SystemMatch,
    UserSelection,
}

/// Discoveryの状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryStatus {
    Success,
    MultipleCandidates,
    NoCandidates,
    EvaluationFailed,
}

/// 最終的に選択されたDiscovery結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub flake_root: PathBuf,
    pub flake_file: PathBuf,

    pub selected_configuration: NixosConfiguration,

    pub selection_method: SelectionMethod,
}

/// 探索全体の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// 探索で見つかった全候補
    pub candidates: Vec<FlakeCandidate>,

    /// 最終的に選択された結果
    pub selected: Option<DiscoveryResult>,

    /// 探索状態
    pub status: DiscoveryStatus,

    /// 探索対象となった場所
    pub search_locations: Vec<PathBuf>,

    /// ユーザーに表示する診断情報
    pub diagnostics: Vec<String>,

    /// 探索実行時刻
    ///
    /// ISO 8601形式の文字列を想定。
    pub generated_at: Option<String>,
}

impl DiscoveryReport {
    pub fn new(status: DiscoveryStatus) -> Self {
        Self {
            candidates: Vec::new(),
            selected: None,
            status,
            search_locations: Vec::new(),
            diagnostics: Vec::new(),
            generated_at: None,
        }
    }
}