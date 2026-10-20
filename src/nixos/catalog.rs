use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

use super::appstream;

const MAX_RESULTS: usize = 40;
const MAX_PAGE_SIZE: usize = 40;
const MAX_CACHED_RESULTS: usize = 5_000;
const MAX_CACHED_QUERIES: usize = 4;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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
    fn get(&mut self, query: &str) -> Option<Arc<CachedSearchResults>> {
        let results = self.entries.get(query)?.clone();
        self.recency.retain(|key| key != query);
        self.recency.push_back(query.to_string());
        Some(results)
    }

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
pub fn search_page(
    query: &str,
    offset: usize,
    limit: usize,
) -> Result<CatalogSearchPage, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(CatalogSearchPage {
            results: Vec::new(),
            next_offset: 0,
            has_more: false,
            results_limited: false,
        });
    }

    let cache_key = query.to_lowercase();
    let cached = search_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&cache_key);

    let cached_results = if let Some(results) = cached {
        results
    } else {
        let output = run_nix_search(query)?;
        let mut ranked = parse_unlimited_results(&output, query)?;
        // Cap per-query memory use even when a broad search matches a very large catalog.
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

/// Find a package by exact attribute ID without truncating the ranked result set first.
pub fn find(id: &str) -> Result<Option<CatalogPackage>, String> {
    if !is_safe_attribute_path(id) {
        return Err(format!("invalid catalog package id: {id}"));
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

    let output = run_nix_search(&requested.join(" "))?;
    parse_find_results(&output, &requested)
}

/// Run the common Nix search command used by discovery and exact package lookup.
fn run_nix_search(query: &str) -> Result<Vec<u8>, String> {
    let terms = query
        .split_whitespace()
        .map(escape_regex)
        .collect::<Vec<_>>();

    let pattern = terms.join("|");
    // Do not rely on the user's global nix.conf to enable the CLI features this command needs.
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "search",
            "nixpkgs",
            "--json",
            "--no-pretty",
            &pattern,
        ])
        .output()
        .map_err(|error| format!("failed to start nix search: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("nix search exited with {}", output.status)
        } else {
            format!("nix search failed: {stderr}")
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

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| text.contains(keyword))
}

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
fn is_safe_attribute_path(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_safe_attribute_segment)
}

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
        assert_eq!(ids, ranked.iter().map(|(_, package)| package.id.as_str()).collect::<Vec<_>>());
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
    fn search_cache_is_bounded_and_evicts_the_least_recently_used_query() {
        let mut cache = SearchCache::default();
        for index in 0..MAX_CACHED_QUERIES {
            cache.insert(format!("query-{index}"), Arc::new(CachedSearchResults::default()));
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
