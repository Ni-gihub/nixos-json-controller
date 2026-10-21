use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::appstream;

const MAX_RESULTS: usize = 40;
const MAX_PAGE_SIZE: usize = 40;
const MAX_CACHED_RESULTS: usize = 5_000;
const MAX_CACHED_QUERIES: usize = 4;
const CATALOG_SCHEMA_VERSION: u32 = 1;
const FULL_CATALOG_PATTERN: &str = ".*";
const MAX_CATALOG_CACHE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_CATALOG_PACKAGES: usize = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogPackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "iconCandidates")]
    pub icon_candidates: Vec<String>,
}

/// One slice of a stable, ranked catalog search result set.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSearchPage {
    pub results: Vec<CatalogPackage>,
    pub next_offset: usize,
    pub has_more: bool,
    pub results_limited: bool,
}

/// Identifies the exact Nixpkgs source resolved by the user's Nix registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CatalogSourceIdentity {
    source: String,
    revision: Option<String>,
    nar_hash: String,
    resolved_url: String,
}

/// Versioned disk cache. Icons are deliberately resolved only for selected results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogCacheEnvelope {
    schema_version: u32,
    source: CatalogSourceIdentity,
    system: String,
    generated_at: u64,
    packages: Vec<CatalogPackage>,
}

#[derive(Debug, Clone)]
struct CatalogIndex {
    source: CatalogSourceIdentity,
    system: String,
    generated_at: u64,
    packages: BTreeMap<String, CatalogPackage>,
    cache_bytes: u64,
}

/// Diagnostics from the most recent freshness check or catalog generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPreparationStatus {
    pub action: String,
    pub source_revision: Option<String>,
    pub source_nar_hash: String,
    pub system: String,
    pub package_count: usize,
    pub generated_at: u64,
    pub cache_path: String,
    pub cache_bytes: u64,
    pub source_check_ms: u64,
    pub cache_load_ms: u64,
    pub generation_ms: u64,
}

#[derive(Default)]
struct CatalogRuntime {
    index: Option<Arc<CatalogIndex>>,
    preparing: bool,
}

/// Return the shared catalog state used by search and preparation requests.
fn catalog_runtime() -> &'static Mutex<CatalogRuntime> {
    static RUNTIME: OnceLock<Mutex<CatalogRuntime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Mutex::new(CatalogRuntime::default()))
}

/// Serialize catalog validation and generation so concurrent requests do not rebuild it.
fn catalog_preparation_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

type RankedCatalogResults = Vec<(u32, CatalogPackage)>;

#[derive(Debug, Default)]
struct CachedSearchResults {
    ranked: RankedCatalogResults,
    truncated: bool,
}

#[derive(Default)]
struct SearchCache {
    entries: HashMap<String, Arc<CachedSearchResults>>,
    recency: VecDeque<String>,
}

impl SearchCache {
    /// Clear all ranked query results after the active catalog source changes.
    fn clear(&mut self) {
        self.entries.clear();
        self.recency.clear();
    }

    /// Return cached ranked results and mark the query as recently used.
    fn get(&mut self, query: &str) -> Option<Arc<CachedSearchResults>> {
        let results = self.entries.get(query)?.clone();
        self.recency.retain(|key| key != query);
        self.recency.push_back(query.to_string());
        Some(results)
    }

    /// Store a query result and evict least-recently-used entries over the cap.
    fn insert(&mut self, query: String, results: Arc<CachedSearchResults>) {
        self.entries.remove(&query);
        self.recency.retain(|key| key != &query);

        while self.entries.len() >= MAX_CACHED_QUERIES {
            let Some(oldest) = self.recency.pop_front() else {
                break;
            };
            self.entries.remove(&oldest);
        }

        self.recency.push_back(query.clone());
        self.entries.insert(query, results);
    }
}

/// Return the bounded, process-local query result cache.
fn search_cache() -> &'static Mutex<SearchCache> {
    static CACHE: OnceLock<Mutex<SearchCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(SearchCache::default()))
}

/// Search the catalog and return the first page of ranked matches.
pub fn search(query: &str) -> Result<Vec<CatalogPackage>, String> {
    Ok(search_page(query, 0, MAX_RESULTS)?.results)
}

/// Search a ranked result set by offset while reusing a bounded, process-local cache.
///
/// Nix search and ranking happen only on a cache miss. Icon candidates are resolved
/// only for the selected page to keep each scroll-triggered request lightweight.
pub fn search_page(query: &str, offset: usize, limit: usize) -> Result<CatalogSearchPage, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(CatalogSearchPage {
            results: Vec::new(),
            next_offset: 0,
            has_more: false,
            results_limited: false,
        });
    }

    let active_index = catalog_index_for_lookup();
    let cache_key = match &active_index {
        Some(index) => format!("catalog:{}:{}", index.source.nar_hash, query.to_lowercase()),
        None => format!("nix:{}", query.to_lowercase()),
    };
    let cached = search_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&cache_key);

    let cached_results = if let Some(results) = cached {
        results
    } else {
        let mut ranked = if let Some(index) = active_index {
            rank_local_matches(&index, query)
        } else {
            let output = run_nix_search(query)?;
            parse_unlimited_results(&output, query)?
        };
        // Keep the existing result bound for broad searches while indexing the full source.
        let truncated = ranked.len() > MAX_CACHED_RESULTS;
        ranked.truncate(MAX_CACHED_RESULTS);
        let results = Arc::new(CachedSearchResults { ranked, truncated });
        search_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(cache_key, results.clone());
        results
    };

    Ok(paginate_ranked_results(
        &cached_results.ranked,
        offset,
        limit,
        cached_results.truncated,
    ))
}

/// Select one bounded page and resolve icons only for packages in that page.
fn paginate_ranked_results(
    ranked: &[(u32, CatalogPackage)],
    offset: usize,
    limit: usize,
    results_limited: bool,
) -> CatalogSearchPage {
    let limit = limit.clamp(1, MAX_PAGE_SIZE);
    let start = offset.min(ranked.len());
    let end = start.saturating_add(limit).min(ranked.len());
    let selected = ranked[start..end].to_vec();
    let results = attach_icon_candidates(selected)
        .into_iter()
        .map(|(_, package)| package)
        .collect();

    CatalogSearchPage {
        results,
        next_offset: end,
        has_more: end < ranked.len(),
        results_limited,
    }
}

/// Check the current Nixpkgs reference, load a matching cache, or atomically generate a new one.
/// This is called on App Store entry and can also be forced by the explicit refresh action.
pub fn prepare_catalog(force_refresh: bool) -> Result<CatalogPreparationStatus, String> {
    prepare_catalog_internal(force_refresh)
}

/// Verify or rebuild the persistent catalog while preventing overlapping preparations.
fn prepare_catalog_internal(force_refresh: bool) -> Result<CatalogPreparationStatus, String> {
    let _serial = catalog_preparation_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    {
        let mut runtime = catalog_runtime()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runtime.preparing = true;
        runtime.index = None;
    }

    let prepared =
        prepare_catalog_inner(force_refresh).map(|(index, status)| (Arc::new(index), status));

    {
        let mut runtime = catalog_runtime()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &prepared {
            Ok((index, _status)) => {
                runtime.index = Some(Arc::clone(index));
            }
            Err(_) => {
                runtime.index = None;
            }
        }
        runtime.preparing = false;
    }

    if prepared.is_ok() {
        search_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    prepared.map(|(_, status)| status)
}

/// Return a ready in-memory catalog without starting or waiting for catalog generation.
fn catalog_index_for_lookup() -> Option<Arc<CatalogIndex>> {
    let runtime = catalog_runtime()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if runtime.preparing {
        return None;
    }
    runtime.index.clone()
}

/// Load a matching disk cache or build, validate, and atomically persist the full catalog.
fn prepare_catalog_inner(
    force_refresh: bool,
) -> Result<(CatalogIndex, CatalogPreparationStatus), String> {
    let cache_path = catalog_cache_path()?;
    let source_check_start = Instant::now();
    let (source, system) = current_nixpkgs_identity()?;
    let source_check_ms = elapsed_ms(source_check_start);

    let load_start = Instant::now();
    if !force_refresh {
        if let Some(index) = load_matching_catalog(&cache_path, &source, &system) {
            let cache_load_ms = elapsed_ms(load_start);
            let status = preparation_status(
                "reused-cache",
                &index,
                &cache_path,
                source_check_ms,
                cache_load_ms,
                0,
            );
            return Ok((index, status));
        }
    }
    let cache_load_ms = elapsed_ms(load_start);

    let generation_start = Instant::now();
    let output = run_nix_search_pattern(FULL_CATALOG_PATTERN)?;
    let parsed = parse_unlimited_results(&output, "")?;
    let mut packages = BTreeMap::new();
    for (_, mut package) in parsed {
        package.icon = None;
        package.icon_candidates.clear();
        packages.entry(package.id.clone()).or_insert(package);
    }
    if packages.is_empty() {
        return Err(
            "full Nixpkgs search returned no packages; keeping the previous cache".to_string(),
        );
    }
    if packages.len() > MAX_CATALOG_PACKAGES {
        return Err(format!(
            "full Nixpkgs search returned too many packages ({})",
            packages.len()
        ));
    }

    // Nix's registry can change while a full enumeration is running. Never label results
    // with an identity different from the source that was actually evaluated.
    let (verified_source, verified_system) = current_nixpkgs_identity()?;
    if source != verified_source || system != verified_system {
        return Err(
            "Nixpkgs source changed while the catalog was being generated; retry the update"
                .to_string(),
        );
    }

    let generated_at = unix_timestamp();
    let envelope = CatalogCacheEnvelope {
        schema_version: CATALOG_SCHEMA_VERSION,
        source: source.clone(),
        system: system.clone(),
        generated_at,
        packages: packages.values().cloned().collect(),
    };
    atomic_write_catalog(&cache_path, &envelope)?;
    let cache_bytes = fs::metadata(&cache_path)
        .map_err(|error| format!("unable to inspect catalog cache: {error}"))?
        .len();
    let generation_ms = elapsed_ms(generation_start);

    let index = CatalogIndex {
        source,
        system,
        generated_at,
        packages,
        cache_bytes,
    };
    let status = preparation_status(
        "generated",
        &index,
        &cache_path,
        source_check_ms,
        cache_load_ms,
        generation_ms,
    );
    Ok((index, status))
}

/// Convert an active catalog and measured timings into the UI-facing status payload.
fn preparation_status(
    action: &str,
    index: &CatalogIndex,
    cache_path: &Path,
    source_check_ms: u64,
    cache_load_ms: u64,
    generation_ms: u64,
) -> CatalogPreparationStatus {
    CatalogPreparationStatus {
        action: action.to_string(),
        source_revision: index.source.revision.clone(),
        source_nar_hash: index.source.nar_hash.clone(),
        system: index.system.clone(),
        package_count: index.packages.len(),
        generated_at: index.generated_at,
        cache_path: cache_path.display().to_string(),
        cache_bytes: index.cache_bytes,
        source_check_ms,
        cache_load_ms,
        generation_ms,
    }
}

/// Resolve the current nixpkgs registry source identity and the host's Nix system.
fn current_nixpkgs_identity() -> Result<(CatalogSourceIdentity, String), String> {
    let metadata = run_nix_command(&["flake", "metadata", "nixpkgs", "--json"])?;
    let metadata: Value = serde_json::from_slice(&metadata)
        .map_err(|error| format!("failed to parse nixpkgs flake metadata: {error}"))?;
    let locked = metadata.get("locked").and_then(Value::as_object);
    let revision = locked
        .and_then(|value| value.get("rev"))
        .and_then(Value::as_str)
        .or_else(|| metadata.get("revision").and_then(Value::as_str))
        .map(str::to_string);
    let nar_hash = locked
        .and_then(|value| value.get("narHash"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "cannot verify the Nixpkgs source: nix flake metadata returned no narHash".to_string()
        })?;
    let resolved_url = metadata
        .get("resolvedUrl")
        .and_then(Value::as_str)
        .or_else(|| metadata.get("url").and_then(Value::as_str))
        .unwrap_or("nixpkgs")
        .to_string();

    let system = run_nix_command(&[
        "eval",
        "--impure",
        "--raw",
        "--expr",
        "builtins.currentSystem",
    ])?;
    let system = String::from_utf8(system)
        .map_err(|error| format!("Nix returned a non-UTF-8 system identifier: {error}"))?
        .trim()
        .to_string();
    if system.is_empty() {
        return Err("Nix returned an empty current system identifier".to_string());
    }

    Ok((
        CatalogSourceIdentity {
            source: "nixpkgs".to_string(),
            revision,
            nar_hash,
            resolved_url,
        },
        system,
    ))
}

/// Return the user cache path, rejecting relative environment-provided roots.
fn catalog_cache_path() -> Result<PathBuf, String> {
    let root = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|home| home.is_absolute())
                .map(|home| home.join(".cache"))
        })
        .ok_or_else(|| {
            "cannot determine a user cache directory (XDG_CACHE_HOME/HOME)".to_string()
        })?;
    Ok(root.join("nxc").join("nixpkgs-catalog-v1.json"))
}

/// Read a bounded cache file and accept it only when source, system, and schema match.
fn load_matching_catalog(
    path: &Path,
    source: &CatalogSourceIdentity,
    system: &str,
) -> Option<CatalogIndex> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_CATALOG_CACHE_BYTES {
        return None;
    }
    let contents = fs::read(path).ok()?;
    let envelope: CatalogCacheEnvelope = serde_json::from_slice(&contents).ok()?;
    catalog_index_from_envelope(envelope, source, system, metadata.len())
}

/// Validate a decoded cache envelope and build a search index without cached icon data.
fn catalog_index_from_envelope(
    envelope: CatalogCacheEnvelope,
    source: &CatalogSourceIdentity,
    system: &str,
    cache_bytes: u64,
) -> Option<CatalogIndex> {
    if envelope.schema_version != CATALOG_SCHEMA_VERSION
        || &envelope.source != source
        || envelope.system != system
        || envelope.packages.is_empty()
        || envelope.packages.len() > MAX_CATALOG_PACKAGES
    {
        return None;
    }

    let mut packages = BTreeMap::new();
    for mut package in envelope.packages {
        if !is_safe_attribute_path(&package.id) || packages.contains_key(&package.id) {
            return None;
        }
        // Cached icons are neither needed nor trusted: resolve them only for visible results.
        package.icon = None;
        package.icon_candidates.clear();
        packages.insert(package.id.clone(), package);
    }

    Some(CatalogIndex {
        source: envelope.source,
        system: envelope.system,
        generated_at: envelope.generated_at,
        packages,
        cache_bytes,
    })
}

/// Persist a catalog cache through a validated temporary file and atomic rename.
fn atomic_write_catalog(path: &Path, envelope: &CatalogCacheEnvelope) -> Result<(), String> {
    atomic_write_catalog_with_limit(path, envelope, MAX_CATALOG_CACHE_BYTES)
}

/// Atomically write a catalog while enforcing a byte limit for production and failure tests.
fn atomic_write_catalog_with_limit(
    path: &Path,
    envelope: &CatalogCacheEnvelope,
    max_bytes: u64,
) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "catalog cache path has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("unable to create catalog cache directory: {error}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("catalog.json");
    let temporary_path = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));

    let write_result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|error| format!("unable to create temporary catalog cache: {error}"))?;
        serde_json::to_writer(&mut file, envelope)
            .map_err(|error| format!("unable to serialize catalog cache: {error}"))?;
        file.flush()
            .map_err(|error| format!("unable to flush catalog cache: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("unable to sync catalog cache: {error}"))?;
        drop(file);
        let temporary_size = fs::metadata(&temporary_path)
            .map_err(|error| format!("unable to inspect temporary catalog cache: {error}"))?
            .len();
        if temporary_size > max_bytes {
            return Err(format!(
                "catalog cache is too large to persist safely ({temporary_size} bytes)"
            ));
        }
        fs::rename(&temporary_path, path)
            .map_err(|error| format!("unable to atomically replace catalog cache: {error}"))?;
        if let Ok(directory) = OpenOptions::new().read(true).open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    write_result
}

/// Rank case-insensitive local ID, name, and description matches using legacy scoring.
fn rank_local_matches(index: &CatalogIndex, query: &str) -> RankedCatalogResults {
    let query_lower = query.to_lowercase();
    let terms = query_lower.split_whitespace().collect::<Vec<_>>();
    let mut results = index
        .packages
        .values()
        .filter(|package| {
            terms.iter().any(|term| {
                package.id.to_lowercase().contains(term)
                    || package.name.to_lowercase().contains(term)
                    || package.description.to_lowercase().contains(term)
            })
        })
        .map(|package| {
            (
                relevance_score(query, &package.id, &package.name, &package.description),
                package.clone(),
            )
        })
        .collect::<Vec<_>>();
    results.sort_by(|(score_a, package_a), (score_b, package_b)| {
        score_b
            .cmp(score_a)
            .then_with(|| {
                package_a
                    .name
                    .to_lowercase()
                    .cmp(&package_b.name.to_lowercase())
            })
            .then_with(|| package_a.id.cmp(&package_b.id))
    });
    results
}

/// Return elapsed milliseconds saturated to the range supported by u64.
fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis().min(u64::MAX as u128) as u64
}

/// Return the current Unix timestamp, defaulting safely if the clock predates the epoch.
fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Find a package by exact attribute ID without truncating the ranked result set first.
pub fn find(id: &str) -> Result<Option<CatalogPackage>, String> {
    if !is_safe_attribute_path(id) {
        return Err(format!("invalid catalog package id: {id}"));
    }

    if let Some(index) = catalog_index_for_lookup() {
        return Ok(index.packages.get(id).cloned().and_then(|package| {
            attach_icon_candidates(vec![(0, package)])
                .into_iter()
                .next()
                .map(|(_, package)| package)
        }));
    }

    let output = run_nix_search(id)?;
    parse_find_result(&output, id)
}

/// Resolve several exact package IDs with one Nix search for the recommendation screen.
pub fn find_many(ids: &[String]) -> Result<Vec<CatalogPackage>, String> {
    let mut requested = Vec::new();
    let mut seen = HashSet::new();

    // Bound work even if an unexpected data file contains an excessive number of IDs.
    for id in ids.iter().take(50) {
        if is_safe_attribute_path(id) && seen.insert(id.as_str()) {
            requested.push(id.clone());
        }
    }

    if requested.is_empty() {
        return Ok(Vec::new());
    }

    if let Some(index) = catalog_index_for_lookup() {
        let requested_set = requested.iter().map(String::as_str).collect::<HashSet<_>>();
        let packages = index
            .packages
            .iter()
            .filter(|(id, _)| requested_set.contains(id.as_str()))
            .map(|(_, package)| (0, package.clone()))
            .collect::<Vec<_>>();
        return Ok(attach_icon_candidates(packages)
            .into_iter()
            .map(|(_, package)| package)
            .collect());
    }

    let output = run_nix_search(&requested.join(" "))?;
    parse_find_results(&output, &requested)
}

/// Run the common Nix search command used by discovery and exact package lookup.
fn run_nix_search(query: &str) -> Result<Vec<u8>, String> {
    let terms = query
        .split_whitespace()
        .map(escape_regex)
        .collect::<Vec<_>>();
    run_nix_search_pattern(&terms.join("|"))
}

/// Run nix search with an already-formed regex. The `.*` pattern enumerates the complete
/// search result space from the same `nixpkgs` registry reference used by normal searches.
fn run_nix_search_pattern(pattern: &str) -> Result<Vec<u8>, String> {
    run_nix_command(&["search", "nixpkgs", "--json", "--no-pretty", pattern])
}

/// Invoke Nix with the features required by this application, preserving stderr on failure.
fn run_nix_command(arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("nix")
        .arg("--extra-experimental-features")
        .arg("nix-command flakes")
        .args(arguments)
        .output()
        .map_err(|error| format!("failed to start nix {}: {error}", arguments.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("nix {} exited with {}", arguments.join(" "), output.status)
        } else {
            format!("nix {} failed: {stderr}", arguments.join(" "))
        });
    }

    Ok(output.stdout)
}

/// Select the exact ID from the full ranked matches, before attaching icons or applying limits.
fn parse_find_result(output: &[u8], id: &str) -> Result<Option<CatalogPackage>, String> {
    let results = parse_unlimited_results(output, id)?;
    let package = results
        .into_iter()
        .find(|(_, package)| package.id == id)
        .map(|(_, package)| package);

    Ok(package.map(|package| {
        attach_icon_candidates(vec![(0, package)])
            .into_iter()
            .next()
            .expect("one package was supplied")
            .1
    }))
}

/// Keep only exact IDs from a multi-package search; substring matches are not recommendations.
fn parse_find_results(output: &[u8], ids: &[String]) -> Result<Vec<CatalogPackage>, String> {
    let requested = ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let query = ids.join(" ");
    let results = parse_unlimited_results(output, &query)?
        .into_iter()
        .filter(|(_, package)| requested.contains(package.id.as_str()))
        .collect::<Vec<_>>();

    Ok(attach_icon_candidates(results)
        .into_iter()
        .map(|(_, package)| package)
        .collect())
}

/// Parse and rank all results without a limit or icon resolution.
fn parse_unlimited_results(
    output: &[u8],
    query: &str,
) -> Result<Vec<(u32, CatalogPackage)>, String> {
    let packages: BTreeMap<String, Value> = serde_json::from_slice(output)
        .map_err(|error| format!("failed to parse nix search output: {error}"))?;

    let mut results = packages
        .into_iter()
        .filter_map(|(attribute, value)| {
            let id = normalize_attribute(&attribute)?;
            if !is_safe_attribute_path(&id) {
                return None;
            }

            let object = value.as_object()?;
            let name = object
                .get("pname")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| id.rsplit('.').next().unwrap_or(&id))
                .to_string();
            let description = object
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let version = object
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let homepage = object
                .get("homepage")
                .and_then(Value::as_str)
                .map(str::to_string);

            let score = relevance_score(query, &id, &name, &description);
            let category = category_for(&name, &description);
            let tags = tags_for(&name, &description, &category);

            Some((
                score,
                CatalogPackage {
                    id,
                    name,
                    description,
                    version,
                    category,
                    tags,
                    homepage,
                    icon: None,
                    icon_candidates: Vec::new(),
                },
            ))
        })
        .collect::<Vec<_>>();

    results.sort_by(|(score_a, package_a), (score_b, package_b)| {
        score_b
            .cmp(score_a)
            .then_with(|| {
                package_a
                    .name
                    .to_lowercase()
                    .cmp(&package_b.name.to_lowercase())
            })
            .then_with(|| package_a.id.cmp(&package_b.id))
    });

    Ok(results)
}

/// Resolve icon candidates only after result selection to avoid doing icon work for discarded rows.
fn attach_icon_candidates(mut results: Vec<(u32, CatalogPackage)>) -> Vec<(u32, CatalogPackage)> {
    for (_, package) in &mut results {
        let id_leaf = package.id.rsplit('.').next().unwrap_or(&package.id);
        let mut icons =
            appstream::icon_candidates_for_package(&[&package.id, id_leaf, &package.name]);
        package.icon = if icons.is_empty() {
            None
        } else {
            Some(icons.remove(0))
        };
        package.icon_candidates = icons;

        if appstream::icon_debug_enabled() {
            eprintln!(
                "[nxc-icon-debug] catalog id={} primary_icon={} additional_candidates={}",
                package.id,
                package.icon.is_some(),
                package.icon_candidates.len()
            );
        }
    }
    results
}

/// Strip the system and packages or legacyPackages prefix from a Nix search attribute.
fn normalize_attribute(attribute: &str) -> Option<String> {
    for prefix in ["legacyPackages.", "packages."] {
        if let Some(rest) = attribute.strip_prefix(prefix) {
            let mut parts = rest.splitn(2, '.');
            let _system = parts.next()?;
            return parts.next().map(str::to_string);
        }
    }

    None
}

/// Score exact, prefix, substring, and individual-term matches for search ordering.
fn relevance_score(query: &str, id: &str, name: &str, description: &str) -> u32 {
    let normalized_query = query.to_lowercase();
    let normalized_id = id.to_lowercase();
    let normalized_name = name.to_lowercase();
    let normalized_description = description.to_lowercase();

    let mut score = 0;
    if normalized_name == normalized_query || normalized_id == normalized_query {
        score += 100;
    }
    if normalized_name.starts_with(&normalized_query)
        || normalized_id.starts_with(&normalized_query)
    {
        score += 80;
    }
    if normalized_name.contains(&normalized_query) || normalized_id.contains(&normalized_query) {
        score += 60;
    }
    if normalized_description.contains(&normalized_query) {
        score += 30;
    }

    for term in normalized_query.split_whitespace() {
        if normalized_name == term {
            score += 40;
        } else if normalized_name.starts_with(term) {
            score += 25;
        } else if normalized_name.contains(term) {
            score += 15;
        } else if normalized_id.contains(term) {
            score += 10;
        } else if normalized_description.contains(term) {
            score += 5;
        }
    }

    score
}

/// Infer the App Store category from package metadata without an external lookup.
fn category_for(name: &str, description: &str) -> String {
    let text = format!("{} {}", name, description).to_lowercase();

    if contains_any(
        &text,
        &["browser", "web browser", "chromium", "firefox", "webkit"],
    ) {
        "ブラウザ".to_string()
    } else if contains_any(
        &text,
        &[
            "compiler",
            "programming",
            "development",
            "debugger",
            "debugging",
            "language server",
            "linter",
            "formatter",
            "sdk",
            "ide",
            "toolchain",
        ],
    ) {
        "開発".to_string()
    } else if contains_any(
        &text,
        &[
            "media player",
            "audio",
            "video",
            "multimedia",
            "music",
            "codec",
        ],
    ) {
        "メディア".to_string()
    } else if contains_any(
        &text,
        &[
            "image editor",
            "image processing",
            "graphics",
            "photo",
            "photography",
            "3d modeling",
            "vector graphics",
        ],
    ) {
        "グラフィックス".to_string()
    } else {
        "その他".to_string()
    }
}

/// Derive searchable tags from package metadata and its inferred category.
fn tags_for(name: &str, description: &str, category: &str) -> Vec<String> {
    let text = format!("{} {}", name, description).to_lowercase();
    let mut tags = vec![category.to_string()];

    let candidates = [
        (
            "オープンソース",
            &["open source", "free software", "free and open source"][..],
        ),
        ("Web", &["web", "http", "browser"][..]),
        ("Rust", &["rust"][..]),
        ("Python", &["python"][..]),
        ("JavaScript", &["javascript", "node.js", "nodejs"][..]),
        ("C/C++", &["c++", "c/c++"][..]),
        ("CLI", &["command line", "cli"][..]),
        ("サーバー", &["server", "daemon"][..]),
        ("ライブラリ", &["library", "libraries"][..]),
    ];

    for (tag, keywords) in candidates {
        if keywords.iter().any(|keyword| text.contains(keyword))
            && !tags.iter().any(|item| item == tag)
        {
            tags.push(tag.to_string());
        }
    }

    tags
}

/// Return whether text contains at least one of the supplied keyword fragments.
fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| text.contains(keyword))
}

/// Escape a user query as a literal-compatible Nix regular expression.
fn escape_regex(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() * 2);
    for character in value.chars() {
        // Hyphens are literal outside regex character classes; \- is invalid in Nix regexes.
        if r#".$^*+?()[]{}|\\"#.contains(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}
/// Validate every segment of a dotted Nix attribute path before passing it to Nix.
fn is_safe_attribute_path(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_safe_attribute_segment)
}

/// Validate one Nix attribute identifier segment and reject reserved keywords.
fn is_safe_attribute_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-') || character == '\''
        })
        && !matches!(
            segment,
            "assert" | "else" | "if" | "in" | "inherit" | "let" | "or" | "rec" | "then" | "with"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_preserves_order_and_has_no_gaps_or_duplicates() {
        let ids = (0..90)
            .map(|index| format!("foo{index:03}"))
            .collect::<Vec<_>>();
        let entries = ids
            .iter()
            .map(|id| (id.as_str(), id.as_str()))
            .collect::<Vec<_>>();
        let output = nix_search_output(&entries);
        let ranked = parse_unlimited_results(&output, "foo").unwrap();

        let first = paginate_ranked_results(&ranked, 0, 40, false);
        let second = paginate_ranked_results(&ranked, first.next_offset, 40, false);
        let third = paginate_ranked_results(&ranked, second.next_offset, 40, false);

        assert_eq!(first.results.len(), 40);
        assert!(first.has_more);
        assert_eq!(second.results.len(), 40);
        assert!(second.has_more);
        assert_eq!(third.results.len(), 10);
        assert!(!third.has_more);
        assert_eq!(third.next_offset, 90);

        let ids = first
            .results
            .iter()
            .chain(second.results.iter())
            .chain(third.results.iter())
            .map(|package| package.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 90);
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 90);
        assert_eq!(
            ids,
            ranked
                .iter()
                .map(|(_, package)| package.id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn paged_response_reports_when_the_cached_result_cap_was_reached() {
        let ids = (0..2)
            .map(|index| format!("foo{index:03}"))
            .collect::<Vec<_>>();
        let entries = ids
            .iter()
            .map(|id| (id.as_str(), id.as_str()))
            .collect::<Vec<_>>();
        let output = nix_search_output(&entries);
        let ranked = parse_unlimited_results(&output, "foo").unwrap();

        let page = paginate_ranked_results(&ranked, 0, 40, true);

        assert!(!page.has_more);
        assert!(page.results_limited);
    }

    #[test]
    fn local_search_matches_case_insensitive_terms_and_ranks_exact_name_first() {
        let packages = [
            CatalogPackage {
                id: "firefox".to_string(),
                name: "Firefox".to_string(),
                description: "Web browser".to_string(),
                version: "1".to_string(),
                category: "ブラウザ".to_string(),
                tags: vec![],
                homepage: None,
                icon: None,
                icon_candidates: vec![],
            },
            CatalogPackage {
                id: "firefox-esr".to_string(),
                name: "Firefox ESR".to_string(),
                description: "Extended support browser".to_string(),
                version: "1".to_string(),
                category: "ブラウザ".to_string(),
                tags: vec![],
                homepage: None,
                icon: None,
                icon_candidates: vec![],
            },
            CatalogPackage {
                id: "chromium".to_string(),
                name: "Chromium".to_string(),
                description: "Firefox compatibility tests".to_string(),
                version: "1".to_string(),
                category: "ブラウザ".to_string(),
                tags: vec![],
                homepage: None,
                icon: None,
                icon_candidates: vec![],
            },
        ];
        let index = CatalogIndex {
            source: CatalogSourceIdentity {
                source: "nixpkgs".to_string(),
                revision: Some("test-revision".to_string()),
                nar_hash: "test-hash".to_string(),
                resolved_url: "nixpkgs".to_string(),
            },
            system: "x86_64-linux".to_string(),
            generated_at: 0,
            packages: packages
                .into_iter()
                .map(|package| (package.id.clone(), package))
                .collect(),
            cache_bytes: 0,
        };

        let ranked = rank_local_matches(&index, "FIREFOX");
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].1.id, "firefox");
        assert!(
            ranked
                .iter()
                .any(|(_, package)| package.id == "firefox-esr")
        );

        let multiple_terms = rank_local_matches(&index, "browser chromium");
        assert!(
            multiple_terms
                .iter()
                .any(|(_, package)| package.id == "chromium")
        );
        assert!(
            multiple_terms
                .iter()
                .any(|(_, package)| package.id == "firefox")
        );
    }

    #[test]
    fn cached_catalog_requires_matching_source_system_schema_and_safe_ids() {
        let source = CatalogSourceIdentity {
            source: "nixpkgs".to_string(),
            revision: Some("rev-a".to_string()),
            nar_hash: "hash-a".to_string(),
            resolved_url: "nixpkgs".to_string(),
        };
        let package = CatalogPackage {
            id: "firefox".to_string(),
            name: "Firefox".to_string(),
            description: "Web browser".to_string(),
            version: "1".to_string(),
            category: "ブラウザ".to_string(),
            tags: vec![],
            homepage: None,
            icon: None,
            icon_candidates: vec![],
        };
        let envelope = CatalogCacheEnvelope {
            schema_version: CATALOG_SCHEMA_VERSION,
            source: source.clone(),
            system: "x86_64-linux".to_string(),
            generated_at: 1,
            packages: vec![package],
        };
        let bytes = serde_json::to_vec(&envelope).unwrap();
        let parsed: CatalogCacheEnvelope = serde_json::from_slice(&bytes).unwrap();
        assert!(
            catalog_index_from_envelope(
                parsed.clone(),
                &source,
                "x86_64-linux",
                bytes.len() as u64,
            )
            .is_some()
        );

        let mut invalid_schema = parsed.clone();
        invalid_schema.schema_version += 1;
        assert!(
            catalog_index_from_envelope(
                invalid_schema,
                &source,
                "x86_64-linux",
                bytes.len() as u64,
            )
            .is_none()
        );

        let mut different_source = source.clone();
        different_source.nar_hash = "hash-b".to_string();
        assert!(
            catalog_index_from_envelope(
                parsed.clone(),
                &different_source,
                "x86_64-linux",
                bytes.len() as u64,
            )
            .is_none()
        );
        assert!(
            catalog_index_from_envelope(
                parsed.clone(),
                &source,
                "aarch64-linux",
                bytes.len() as u64,
            )
            .is_none()
        );

        let mut invalid = parsed;
        invalid.packages[0].id = "firefox; builtins.abort".to_string();
        assert!(
            catalog_index_from_envelope(invalid, &source, "x86_64-linux", bytes.len() as u64,)
                .is_none()
        );
    }

    #[test]
    fn corrupted_disk_cache_is_rejected_and_valid_cache_can_be_reloaded() {
        let root = std::env::temp_dir().join(format!(
            "nxc-catalog-corrupt-cache-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("catalog.json");
        let source = CatalogSourceIdentity {
            source: "nixpkgs".to_string(),
            revision: Some("test-revision".to_string()),
            nar_hash: "test-nar-hash".to_string(),
            resolved_url: "nixpkgs".to_string(),
        };

        std::fs::write(&path, b"{invalid json").unwrap();
        assert!(load_matching_catalog(&path, &source, "x86_64-linux").is_none());

        let envelope = CatalogCacheEnvelope {
            schema_version: CATALOG_SCHEMA_VERSION,
            source: source.clone(),
            system: "x86_64-linux".to_string(),
            generated_at: 42,
            packages: vec![CatalogPackage {
                id: "firefox".to_string(),
                name: "Firefox".to_string(),
                description: "Web browser".to_string(),
                version: "1".to_string(),
                category: "ブラウザ".to_string(),
                tags: vec![],
                homepage: None,
                icon: Some("untrusted-cache-icon".to_string()),
                icon_candidates: vec!["also-untrusted".to_string()],
            }],
        };
        atomic_write_catalog(&path, &envelope).unwrap();

        let loaded =
            load_matching_catalog(&path, &source, "x86_64-linux").expect("matching cache loads");
        let package = loaded.packages.get("firefox").unwrap();
        assert_eq!(loaded.generated_at, 42);
        assert!(package.icon.is_none());
        assert!(package.icon_candidates.is_empty());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_atomic_cache_update_preserves_the_previous_file() {
        let root = std::env::temp_dir().join(format!(
            "nxc-catalog-atomic-write-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("catalog.json");
        let previous_contents = b"previous-valid-cache".to_vec();
        std::fs::write(&path, &previous_contents).unwrap();

        let envelope = CatalogCacheEnvelope {
            schema_version: CATALOG_SCHEMA_VERSION,
            source: CatalogSourceIdentity {
                source: "nixpkgs".to_string(),
                revision: Some("test-revision".to_string()),
                nar_hash: "test-nar-hash".to_string(),
                resolved_url: "nixpkgs".to_string(),
            },
            system: "x86_64-linux".to_string(),
            generated_at: 42,
            packages: vec![CatalogPackage {
                id: "firefox".to_string(),
                name: "Firefox".to_string(),
                description: "Web browser".to_string(),
                version: "1".to_string(),
                category: "ブラウザ".to_string(),
                tags: vec![],
                homepage: None,
                icon: None,
                icon_candidates: vec![],
            }],
        };

        assert!(atomic_write_catalog_with_limit(&path, &envelope, 1).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), previous_contents);
        assert_eq!(
            std::fs::read_dir(&root).unwrap().count(),
            1,
            "a failed write should not leave a temporary file"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn search_cache_is_bounded_and_evicts_the_least_recently_used_query() {
        let mut cache = SearchCache::default();
        for index in 0..MAX_CACHED_QUERIES {
            cache.insert(
                format!("query-{index}"),
                Arc::new(CachedSearchResults::default()),
            );
        }
        assert!(cache.get("query-0").is_some());

        cache.insert(
            "query-new".to_string(),
            Arc::new(CachedSearchResults::default()),
        );

        assert_eq!(cache.entries.len(), MAX_CACHED_QUERIES);
        assert!(cache.entries.contains_key("query-0"));
        assert!(!cache.entries.contains_key("query-1"));
        assert!(cache.entries.contains_key("query-new"));
    }

    #[test]
    fn escape_regex_leaves_hyphens_unescaped_for_nix() {
        assert_eq!(escape_regex("obs-studio"), "obs-studio");
        assert_eq!(escape_regex("telegram-desktop"), "telegram-desktop");
        assert_eq!(escape_regex("kdePackages.okular"), "kdePackages\\.okular");
        assert_eq!(escape_regex("a+b"), "a\\+b");
    }

    #[test]
    fn normalizes_nix_search_attribute() {
        assert_eq!(
            normalize_attribute("legacyPackages.x86_64-linux.firefox"),
            Some("firefox".to_string())
        );
        assert_eq!(
            normalize_attribute("legacyPackages.x86_64-linux.python3Packages.requests"),
            Some("python3Packages.requests".to_string())
        );
    }

    #[test]
    #[ignore = "requires network access and enumerates the full Nixpkgs catalog"]
    fn full_catalog_benchmark_compares_legacy_and_local_search() {
        let (source_before, system_before) =
            current_nixpkgs_identity().expect("resolve current Nixpkgs source");
        let queries = ["firefox", "vscode", "python", "http", "rust compiler"];
        let mut baseline = HashMap::new();

        println!(
            "catalog-benchmark source_nar_hash={} system={}",
            source_before.nar_hash, system_before
        );

        for query in queries {
            let started = Instant::now();
            let output = run_nix_search(query).expect("legacy nix search succeeds");
            let mut ranked =
                parse_unlimited_results(&output, query).expect("legacy search JSON parses");
            let matched_count = ranked.len();
            let truncated = matched_count > MAX_CACHED_RESULTS;
            ranked.truncate(MAX_CACHED_RESULTS);
            let page = paginate_ranked_results(&ranked, 0, MAX_RESULTS, truncated);
            let elapsed_ms = elapsed_ms(started);
            let ids = page
                .results
                .iter()
                .map(|package| package.id.clone())
                .collect::<Vec<_>>();

            assert!(
                !ids.is_empty(),
                "legacy search returned no results for {query}"
            );
            println!(
                "catalog-benchmark legacy query={query:?} elapsed_ms={elapsed_ms} matched={matched_count} first_page={}",
                ids.len()
            );
            baseline.insert(query, (ids, matched_count, elapsed_ms));
        }

        let generated = prepare_catalog(true).expect("generate a verified full catalog");
        assert_eq!(
            generated.source_nar_hash, source_before.nar_hash,
            "baseline and catalog must use the same resolved Nixpkgs source"
        );
        assert_eq!(
            generated.system, system_before,
            "baseline and catalog must use the same system"
        );
        println!(
            "catalog-benchmark generation packages={} cache_bytes={} source_check_ms={} generation_ms={}",
            generated.package_count,
            generated.cache_bytes,
            generated.source_check_ms,
            generated.generation_ms
        );

        let loaded = prepare_catalog(false).expect("reload the verified disk catalog");
        assert_eq!(loaded.action, "reused-cache");
        println!(
            "catalog-benchmark reload packages={} cache_bytes={} source_check_ms={} cache_load_ms={}",
            loaded.package_count, loaded.cache_bytes, loaded.source_check_ms, loaded.cache_load_ms
        );

        for query in queries {
            let started = Instant::now();
            let local = search_page(query, 0, MAX_RESULTS).expect("local search succeeds");
            let cold_elapsed_ms = elapsed_ms(started);
            let local_ids = local
                .results
                .iter()
                .map(|package| package.id.clone())
                .collect::<Vec<_>>();
            assert!(
                !local_ids.is_empty(),
                "local search returned no results for {query}"
            );

            let (legacy_ids, legacy_count, legacy_elapsed_ms) = &baseline[query];
            let legacy_id_set = legacy_ids.iter().collect::<HashSet<_>>();
            let overlap = local_ids
                .iter()
                .filter(|id| legacy_id_set.contains(id))
                .count();

            let warm_started = Instant::now();
            let warm = search_page(query, 0, MAX_RESULTS).expect("cached local search succeeds");
            let warm_elapsed_ms = elapsed_ms(warm_started);
            assert_eq!(
                local_ids,
                warm.results
                    .iter()
                    .map(|package| package.id.clone())
                    .collect::<Vec<_>>(),
                "cached local search must preserve its first result page"
            );

            assert_eq!(
                local_ids.as_slice(),
                legacy_ids.as_slice(),
                "local search first page must match legacy nix search for query {query:?}"
            );

            println!(
                "catalog-benchmark compare query={query:?} legacy_ms={legacy_elapsed_ms} local_first_ms={cold_elapsed_ms} local_cached_ms={warm_elapsed_ms} legacy_matched={legacy_count} local_page={} overlap_with_legacy_first_page={overlap}/{} legacy_top3={:?} local_top3={:?}",
                local_ids.len(),
                legacy_ids.len(),
                legacy_ids.iter().take(3).collect::<Vec<_>>(),
                local_ids.iter().take(3).collect::<Vec<_>>()
            );
        }
    }

    fn nix_search_output(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut output = serde_json::Map::new();
        for (id, name) in entries {
            output.insert(
                format!("legacyPackages.x86_64-linux.{id}"),
                serde_json::json!({
                    "pname": name,
                    "description": "",
                    "version": "1.0"
                }),
            );
        }
        serde_json::to_vec(&output).unwrap()
    }

    #[test]
    fn find_checks_exact_id_before_the_ordinary_search_limit() {
        let ids = (0..45)
            .map(|index| format!("a{index:02}.foo"))
            .chain(std::iter::once("foo".to_string()))
            .collect::<Vec<_>>();
        let entries = ids
            .iter()
            .map(|id| (id.as_str(), "foo"))
            .collect::<Vec<_>>();
        let output = nix_search_output(&entries);

        let ranked = parse_unlimited_results(&output, "foo").unwrap();
        let limited = paginate_ranked_results(&ranked, 0, MAX_RESULTS, false).results;
        assert_eq!(limited.len(), MAX_RESULTS);
        assert!(!limited.iter().any(|package| package.id == "foo"));

        let found = parse_find_result(&output, "foo").unwrap().unwrap();
        assert_eq!(found.id, "foo");
    }

    #[test]
    fn finds_exact_hyphenated_and_nested_attribute_ids() {
        let hyphenated = nix_search_output(&[
            ("google-chrome-beta", "chrome"),
            ("google-chrome", "chrome"),
        ]);
        assert_eq!(
            parse_find_result(&hyphenated, "google-chrome")
                .unwrap()
                .unwrap()
                .id,
            "google-chrome"
        );

        let nested = nix_search_output(&[
            ("python3Packages.requests2", "requests"),
            ("python3Packages.requests", "requests"),
        ]);
        assert_eq!(
            parse_find_result(&nested, "python3Packages.requests")
                .unwrap()
                .unwrap()
                .id,
            "python3Packages.requests"
        );
    }

    #[test]
    fn ranks_exact_name_above_description_match() {
        let exact = relevance_score("rust", "rust", "rust", "Programming language");
        let description = relevance_score("rust", "foo", "foo", "Rust development tool");

        assert!(exact > description);
    }

    #[test]
    fn accepts_nested_nix_attribute_paths() {
        assert!(is_safe_attribute_path("python3Packages.requests"));
        assert!(is_safe_attribute_path("llvmPackages_20.clang"));
    }

    #[test]
    fn rejects_expression_injection() {
        assert!(!is_safe_attribute_path("firefox;builtins.abort"));
        assert!(!is_safe_attribute_path("firefox $(touch /tmp/pwned)"));
    }

    #[test]
    fn finds_only_exact_ids_from_a_multi_package_search() {
        let output = nix_search_output(&[
            ("firefox", "firefox"),
            ("firefox-esr", "firefox"),
            ("chromium", "chromium"),
            ("epiphany", "epiphany"),
        ]);
        let ids = vec!["firefox".to_string(), "chromium".to_string()];
        let found = parse_find_results(&output, &ids).unwrap();
        let found_ids = found
            .iter()
            .map(|package| package.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(found_ids, vec!["chromium", "firefox"]);
    }

    #[test]
    fn generates_coarse_metadata_without_manual_catalog_entries() {
        assert_eq!(
            category_for("rust-analyzer", "Language server for Rust"),
            "開発"
        );
        assert!(
            tags_for("rust-analyzer", "Language server for Rust", "開発")
                .contains(&"Rust".to_string())
        );
    }
}
