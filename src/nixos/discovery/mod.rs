pub mod candidate;
pub mod context;
pub mod inspection;
pub mod nix;
pub mod result;
pub mod search;
pub mod selector;
pub mod state;

pub use candidate::{CandidateScore, DiscoverySource, Evidence, FlakeCandidate};

pub use context::DiscoveryContext;

pub use inspection::{
    EnvironmentMatch, EvaluationErrorCategory, FilesystemInspection, FlakeOutputs,
    InspectionResult, NixEvaluation, NixEvaluationError, NixosConfiguration,
};

pub use nix::{
    NixCommandError, NixFlakeMetadata, NixFlakeOutput, NixFlakeShow, evaluate_flake,
    evaluate_nixos_configuration, inspect_flake, show_flake,
};

pub use result::{
    DiscoveryMetadata, DiscoveryReport, DiscoveryResult, DiscoveryStatus, FileMetadata,
    SelectionMethod,
};

pub use search::{SearchOptions, discover_candidates, inspect_candidates};

pub use selector::select_candidate;

pub use state::{discovery_exists, load_discovery, save_discovery, state_path, validate_discovery};
