use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::inspection::InspectionResult;

/// Flakeの探索元
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoverySource {
    Cli,
    ConfigFile,
    EnvironmentVariable,
    HomeDirectory,
    GitRepository,
    SystemHint,
}

/// 候補から得られた証拠
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Evidence {
    FlakeFile,
    FlakeLock,
    GitRepository,
    HostsDirectory,
    ModulesDirectory,
    HomeDirectory,
    SystemDirectory,
    ConfigurationNix,
    HardwareConfigurationNix,
    NixosConfigurationsText,
}

/// 探索中に発見されたFlake候補
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeCandidate {
    /// Flakeのルートディレクトリ
    pub flake_root: PathBuf,

    /// flake.nixのパス
    pub flake_file: PathBuf,

    /// この候補が発見された経路
    pub sources: Vec<DiscoverySource>,

    /// 候補から得られた証拠
    pub evidence: Vec<Evidence>,

    /// 検査結果
    pub inspection: Option<InspectionResult>,
}

impl FlakeCandidate {
    pub fn new(flake_root: PathBuf, flake_file: PathBuf) -> Self {
        Self {
            flake_root,
            flake_file,
            sources: Vec::new(),
            evidence: Vec::new(),
            inspection: None,
        }
    }
}