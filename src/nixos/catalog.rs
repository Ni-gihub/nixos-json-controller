use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

const MAX_RESULTS: usize = 40;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatalogPackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
}

pub fn search(query: &str) -> Result<Vec<CatalogPackage>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let terms = query
        .split_whitespace()
        .map(escape_regex)
        .collect::<Vec<_>>();

    let pattern = terms.join("|");
    let output = Command::new("nix")
        .args(["search", "nixpkgs", "--json", "--no-pretty", &pattern])
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

    parse_results(&output.stdout, query)
}

pub fn find(id: &str) -> Result<Option<CatalogPackage>, String> {
    if !is_safe_attribute_path(id) {
        return Err(format!("invalid catalog package id: {id}"));
    }

    let output = Command::new("nix")
        .args([
            "search",
            "nixpkgs",
            "--json",
            "--no-pretty",
            &format!("^{}$", regex_escape_attribute(id)),
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

    Ok(parse_results(&output.stdout, id)?
        .into_iter()
        .find(|package| package.id == id))
}

fn parse_results(output: &[u8], query: &str) -> Result<Vec<CatalogPackage>, String> {
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

    results.truncate(MAX_RESULTS);
    Ok(results.into_iter().map(|(_, package)| package).collect())
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
        if r#".$^*+?()[]{}|\\-"#.contains(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn regex_escape_attribute(value: &str) -> String {
    value
        .split('.')
        .map(escape_regex)
        .collect::<Vec<_>>()
        .join(r#"\."#)
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
