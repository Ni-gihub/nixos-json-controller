use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::candidate::FlakeCandidate;
use super::inspection::NixosConfiguration;

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

/// Discovery時点のファイル状態。
///
/// 通常操作時のstale判定に利用する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileMetadata {
    /// ファイルサイズ
    pub size: u64,

    /// 最終更新時刻。
    ///
    /// UNIX epochからのナノ秒。
    pub modified_ns: u128,
}

impl FileMetadata {
    pub fn from_path(
        path: &std::path::Path,
    ) -> Result<Self, String> {
        let metadata =
            std::fs::metadata(path)
                .map_err(|e| {
                    format!(
                        "failed to read file metadata {}: {}",
                        path.display(),
                        e
                    )
                })?;

        let modified =
            metadata
                .modified()
                .map_err(|e| {
                    format!(
                        "failed to read modification time {}: {}",
                        path.display(),
                        e
                    )
                })?;

        let modified_ns =
            modified
                .duration_since(UNIX_EPOCH)
                .map_err(|e| {
                    format!(
                        "invalid modification time {}: {}",
                        path.display(),
                        e
                    )
                })?
                .as_nanos();

        Ok(Self {
            size: metadata.len(),
            modified_ns,
        })
    }
}

/// Discovery保存時の状態。
///
/// これは「このDiscovery結果が、いつ・どのファイル状態を
/// 前提として作られたか」を記録する。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryMetadata {
    /// Discovery実行時刻。
    pub discovered_at: u64,

    /// Discovery時点のflake.nix。
    pub flake_nix: FileMetadata,

    /// Discovery時点のflake.lock。
    ///
    /// lock fileが存在しないFlakeではNone。
    pub flake_lock: Option<FileMetadata>,
}

impl DiscoveryMetadata {
    /// 現在のファイル状態からDiscoveryMetadataを作る。
    pub fn capture(
        flake_root: &std::path::Path,
    ) -> Result<Self, String> {
        let flake_file =
            flake_root.join("flake.nix");

        let flake_nix =
            FileMetadata::from_path(
                &flake_file,
            )?;

        let flake_lock_path =
            flake_root.join("flake.lock");

        let flake_lock =
            if flake_lock_path.is_file() {
                Some(
                    FileMetadata::from_path(
                        &flake_lock_path,
                    )?
                )
            } else {
                None
            };

        let discovered_at =
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| {
                    format!(
                        "failed to determine discovery time: {}",
                        e
                    )
                })?
                .as_secs();

        Ok(Self {
            discovered_at,
            flake_nix,
            flake_lock,
        })
    }
}

/// 最終的に選択されたDiscovery結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub flake_root: PathBuf,
    pub flake_file: PathBuf,

    pub selected_configuration: NixosConfiguration,

    pub selection_method: SelectionMethod,

    /// Discovery時点の状態。
    ///
    /// 古いdiscovery.jsonとの互換性のためoptional。
    #[serde(default)]
    pub metadata: Option<DiscoveryMetadata>,
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