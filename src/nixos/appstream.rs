use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

static INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
static STOCK_ICON_INDEX: OnceLock<HashMap<String, Vec<PathBuf>>> = OnceLock::new();

/// Resolve the first usable AppStream icon for the supplied package aliases.
pub fn icon_for_package(candidates: &[&str]) -> Option<String> {
    let index = INDEX.get_or_init(load_index);
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    for candidate in candidates {
        let icon_entries = index
            .get(*candidate)
            .or_else(|| index.get(&candidate.to_lowercase()));

        let Some(icon_entries) = icon_entries else {
            continue;
        };

        for icon_entry in icon_entries {
            if let Ok(icons) = cache.lock() {
                if let Some(cached) = icons.get(icon_entry) {
                    if cached.is_some() {
                        return cached.clone();
                    }
                    continue;
                }
            }

            let icon = load_icon(icon_entry);
            if let Ok(mut icons) = cache.lock() {
                icons.insert(icon_entry.clone(), icon.clone());
            }
            if icon.is_some() {
                return icon;
            }
        }
    }

    None
}

/// Load AppStream component aliases and retain every icon candidate for fallback resolution.
fn load_index() -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();

    for (xml_path, icon_root) in find_xml() {
        let Ok(output) = Command::new("gzip")
            .args(["-dc"])
            .arg(&xml_path)
            .output()
        else {
            eprintln!("failed to start gzip for AppStream data {}", xml_path.display());
            continue;
        };

        if !output.status.success() {
            eprintln!(
                "failed to decompress AppStream data {}: status={}, stderr={}",
                xml_path.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            continue;
        }

        for (package, icons) in parse_index(&String::from_utf8_lossy(&output.stdout)) {
            let entries = index.entry(package).or_default();
            for icon in icons {
                let source = if icon.starts_with("remote|") || icon.starts_with("stock|") {
                    icon.clone()
                } else {
                    format!("cached|{}|{}", icon_root.display(), icon)
                };
                merge_icon_candidate(entries, source);
            }
        }
    }

    index
}

fn find_xml() -> Vec<(PathBuf, PathBuf)> {
    let mut roots = Vec::new();

    if let Ok(root) = env::var("NXC_APPSTREAM_DATA") {
        roots.push(PathBuf::from(root));
    }

    roots.extend([
        PathBuf::from("/run/current-system/sw"),
        PathBuf::from("/usr"),
    ]);

    roots
        .into_iter()
        .flat_map(|root| {
            ["nixos-unstable", "nixos-unstable-unfree"]
                .into_iter()
                .filter_map(move |section| {
                    let xml = root.join("share/swcatalog/xml").join(format!("{section}.xml.gz"));
                    let icons = root.join("share/swcatalog/icons").join(section);
                    xml.is_file().then_some((xml, icons))
                })
        })
        .collect()
}

/// Merge icon candidates in order: cached/local file, stock theme icon, then remote URL.
fn merge_icon_candidate(entries: &mut Vec<String>, candidate: String) {
    if entries.contains(&candidate) {
        return;
    }

    let candidate_priority = icon_source_priority(&candidate);
    let insert_at = entries
        .iter()
        .position(|entry| icon_source_priority(entry) > candidate_priority)
        .unwrap_or(entries.len());
    entries.insert(insert_at, candidate);
}

fn icon_source_priority(entry: &str) -> u8 {
    if entry.starts_with("remote|") {
        2
    } else if entry.starts_with("stock|") {
        1
    } else {
        0
    }
}

/// Parse AppStream components into package aliases and ordered icon candidates.
fn parse_index(xml: &str) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    let mut offset = 0;

    while let Some(start) = xml[offset..].find("<component ") {
        let start = offset + start;
        let Some(end_rel) = xml[start..].find("</component>") else {
            break;
        };
        let end = start + end_rel + "</component>".len();
        let component = &xml[start..end];

        let icons = icon_sources(component);
        if icons.is_empty() {
            offset = end;
            continue;
        }

        let mut packages = Vec::new();

        if let Some(pkgname) = element_text(component, "pkgname") {
            packages.extend(pkgname.split_whitespace().map(str::to_string));
        }

        if let Some(id) = element_text(component, "id") {
            packages.push(id.clone());
            if let Some(leaf) = id.rsplit('.').next() {
                packages.push(leaf.to_string());
            }
        }

        for launchable in element_texts(component, "launchable") {
            if let Some(desktop_id) = launchable.strip_suffix(".desktop") {
                packages.push(desktop_id.to_string());
            }
        }

        for package in packages {
            let aliases = [package.clone(), package.to_lowercase()];
            for alias in aliases {
                let entries = index.entry(alias).or_default();
                for icon in &icons {
                    merge_icon_candidate(entries, icon.clone());
                }
            }
        }

        offset = end;
    }

    index
}

/// Extract cached, stock-theme, and absolute remote icon references from one AppStream component.
fn icon_sources(input: &str) -> Vec<String> {
    let mut cached_icons = Vec::new();
    let mut stock_icons = Vec::new();
    let mut remote_icons = Vec::new();
    let mut offset = 0;

    while let Some(relative_start) = input[offset..].find("<icon") {
        let start = offset + relative_start;
        let Some(tag_end_rel) = input[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end_rel;
        let tag = &input[start..=tag_end];

        if tag.starts_with("<icon ") || tag.starts_with("<icon>") {
            let icon_type = element_attribute(tag, "icon", "type");
            let content_start = tag_end + 1;
            if let Some(close_rel) = input[content_start..].find("</icon>") {
                let value = input[content_start..content_start + close_rel].trim();
                if !value.is_empty() {
                    let value = decode_xml_entities(value);
                    match icon_type.as_deref() {
                        Some("cached") | None => cached_icons.push(value),
                        Some("stock") => stock_icons.push(format!("stock|{value}")),
                        Some("remote")
                            if value.starts_with("https://") || value.starts_with("http://") =>
                        {
                            remote_icons.push(format!("remote|{value}"));
                        }
                        _ => {}
                    }
                }
                offset = content_start + close_rel + "</icon>".len();
                continue;
            }
        }

        offset = tag_end + 1;
    }

    cached_icons.extend(stock_icons);
    cached_icons.extend(remote_icons);
    cached_icons
}

/// Decode the XML entities that can occur in AppStream text nodes.
fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Return the first non-empty text value for an XML element name.
fn element_text(input: &str, element: &str) -> Option<String> {
    element_texts(input, element).into_iter().next()
}

/// Return all non-empty text values for an XML element name.
fn element_texts(input: &str, element: &str) -> Vec<String> {
    let open = format!("<{element}");
    let close = format!("</{element}>");
    let mut values = Vec::new();
    let mut offset = 0;

    while let Some(relative_start) = input[offset..].find(&open) {
        let start = offset + relative_start;
        let Some(content_start_rel) = input[start..].find('>') else {
            break;
        };
        let content_start = start + content_start_rel + 1;
        let Some(content_end_rel) = input[content_start..].find(&close) else {
            break;
        };
        let content_end = content_start + content_end_rel;
        let value = input[content_start..content_end].trim();
        if !value.is_empty() {
            values.push(value.to_string());
        }
        offset = content_end + close.len();
    }

    values
}

/// Read a quoted attribute from the first matching XML start tag.
fn element_attribute(input: &str, element: &str, attribute: &str) -> Option<String> {
    let open = format!("<{element}");
    let start = input.find(&open)?;
    let tag_end = input[start..].find('>')? + start;
    let tag = &input[start..tag_end];
    let marker = format!("{attribute}=\"");
    let value_start = tag.find(&marker)? + marker.len();
    let value_end = tag[value_start..].find('"')? + value_start;
    Some(tag[value_start..value_end].to_string())
}

/// Load an AppStream icon from cached data, the system icon theme, or a remote URL.
fn load_icon(entry: &str) -> Option<String> {
    if let Some(url) = entry.strip_prefix("remote|") {
        return Some(url.to_string());
    }

    if let Some(icon_name) = entry.strip_prefix("stock|") {
        return load_stock_icon(icon_name);
    }

    let entry = entry.strip_prefix("cached|").unwrap_or(entry);
    let (root, icon_name) = entry.split_once('|')?;
    let root = Path::new(root);

    let mut names = vec![icon_name.to_string()];
    if let Some(name) = icon_name.strip_suffix(".desktop") {
        names.push(name.to_string());
    }

    // Prefer scalable/high-resolution assets so the App Store does not upscale
    // a small cached icon when a better source is already available.
    let preferred_sizes = [
        "scalable",
        "256x256",
        "128x128@2",
        "128x128",
        "96x96@2",
        "96x96",
        "64x64@2",
        "64x64",
        "48x48@2",
        "48x48",
    ];

    for size in preferred_sizes {
        for name in &names {
            if let Some(data_url) = read_icon(&root.join(size).join(name)) {
                return Some(data_url);
            }
        }
    }

    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        for name in &names {
            if let Some(data_url) = read_icon(&path.join(name)) {
                return Some(data_url);
            }
        }
    }

    None
}

/// Resolve stock icon names against icon themes available to the current user/system.
///
/// XDG data directories are included because profile-installed icon themes may live outside
/// `/run/current-system/sw`. Icon files are converted to data URLs so the web frontend does
/// not need direct filesystem access.
fn load_stock_icon(icon_name: &str) -> Option<String> {
    let index = STOCK_ICON_INDEX.get_or_init(build_stock_icon_index);
    let paths = index.get(&icon_name.to_lowercase())?;

    paths.iter().find_map(|path| read_icon(path))
}

fn build_stock_icon_index() -> HashMap<String, Vec<PathBuf>> {
    build_stock_icon_index_from_roots(&stock_icon_roots())
}

fn build_stock_icon_index_from_roots(roots: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let mut index: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut visited = HashSet::new();

    for root in roots {
        index_stock_icon_dir(root, &mut index, &mut visited, 0);
    }

    for paths in index.values_mut() {
        paths.sort_by_key(|path| (stock_icon_path_priority(path), path.clone()));
    }

    index
}

fn stock_icon_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(data_dirs) = env::var("XDG_DATA_DIRS") {
        for data_dir in data_dirs.split(':').filter(|value| !value.is_empty()) {
            let data_dir = PathBuf::from(data_dir);
            roots.push(data_dir.join("icons"));
            roots.push(data_dir.join("pixmaps"));
        }
    }

    if let Ok(home) = env::var("HOME") {
        let home = PathBuf::from(home);
        roots.extend([
            home.join(".local/share/icons"),
            home.join(".icons"),
            home.join(".local/share/pixmaps"),
            home.join(".nix-profile/share/icons"),
            home.join(".nix-profile/share/pixmaps"),
        ]);
    }

    roots.extend([
        PathBuf::from("/run/current-system/sw/share/icons"),
        PathBuf::from("/run/current-system/sw/share/pixmaps"),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/share/pixmaps"),
        PathBuf::from("/usr/local/share/icons"),
        PathBuf::from("/usr/local/share/pixmaps"),
    ]);

    let mut seen = HashSet::new();
    roots
        .into_iter()
        .filter(|root| root.is_dir())
        .filter(|root| {
            let identity = fs::canonicalize(root).unwrap_or_else(|_| root.clone());
            seen.insert(identity)
        })
        .collect()
}

fn index_stock_icon_dir(
    directory: &Path,
    index: &mut HashMap<String, Vec<PathBuf>>,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) {
    // Icon theme directories are shallow; cap traversal and guard against symlink cycles.
    if depth > 12 {
        return;
    }

    let Ok(metadata) = fs::metadata(directory) else {
        return;
    };
    if !metadata.is_dir() {
        return;
    }

    let identity = fs::canonicalize(directory).unwrap_or_else(|_| directory.to_path_buf());
    if !visited.insert(identity) {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            index_stock_icon_dir(&path, index, visited, depth + 1);
            continue;
        }

        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !matches!(extension.to_ascii_lowercase().as_str(), "png" | "svg" | "jpg" | "jpeg") {
            continue;
        }

        let Some(name) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        index.entry(name.to_lowercase()).or_default().push(path);
    }
}

fn stock_icon_path_priority(path: &Path) -> (u8, u8) {
    let normalized = path.to_string_lossy().to_lowercase();
    let size_priority = if normalized.contains("/scalable/") {
        0
    } else if normalized.contains("/256x256/") || normalized.contains("/256x256@2/") {
        1
    } else if normalized.contains("/128x128/") || normalized.contains("/128x128@2/") {
        2
    } else if normalized.contains("/96x96/") || normalized.contains("/96x96@2/") {
        3
    } else if normalized.contains("/64x64/") || normalized.contains("/64x64@2/") {
        4
    } else if normalized.contains("/48x48/") || normalized.contains("/48x48@2/") {
        5
    } else {
        6
    };

    let category_priority = if normalized.contains("/apps/") {
        0
    } else if normalized.contains("/mimetypes/") {
        1
    } else {
        2
    };

    (size_priority, category_priority)
}

/// Read a supported local image and encode it as a data URL.
fn read_icon(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mime = match path.extension().and_then(|extension| extension.to_str()) {
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => return None,
    };

    Some(format!("data:{mime};base64,{}", base64_encode(&bytes)))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);

        output.push(TABLE[(a >> 2) as usize] as char);
        output.push(TABLE[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(TABLE[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(TABLE[(c & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_cached_icons_and_aliases() {
        let xml = r#"
            <component type="desktop-application">
              <id>org.mozilla.Firefox</id>
              <pkgname>firefox</pkgname>
              <launchable type="desktop-id">firefox.desktop</launchable>
              <icon type="cached">org.mozilla.firefox-64.png</icon>
              <icon type="cached">org.mozilla.firefox-128.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        let expected = vec![
            "org.mozilla.firefox-64.png".to_string(),
            "org.mozilla.firefox-128.png".to_string(),
        ];
        assert_eq!(index.get("firefox"), Some(&expected));
        assert_eq!(index.get("org.mozilla.Firefox"), Some(&expected));
        assert_eq!(index.get("org.mozilla.firefox"), Some(&expected));
    }

    #[test]
    fn merges_cached_icons_before_remote_icons_across_components() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/remote.png</icon>
            </component>
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="cached">example.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec![
                "example.png".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ])
        );
    }

    #[test]
    fn merge_icon_candidate_keeps_cached_first() {
        let mut entries = vec![
            "remote|https://example.com/remote.png".to_string(),
        ];

        merge_icon_candidate(&mut entries, "cached|/icons|example.png".to_string());

        assert_eq!(
            entries,
            vec![
                "cached|/icons|example.png".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ]
        );
    }

    #[test]
    fn parses_stock_icons_and_keeps_cached_first_remote_last() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/remote.png</icon>
              <icon type="stock">applications-internet</icon>
              <icon type="cached">example.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec![
                "example.png".to_string(),
                "stock|applications-internet".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ])
        );
    }

    #[test]
    fn merge_icon_candidate_keeps_stock_between_cached_and_remote() {
        let mut entries = vec![
            "remote|https://example.com/remote.png".to_string(),
            "stock|applications-internet".to_string(),
        ];

        merge_icon_candidate(&mut entries, "cached|/icons|example.png".to_string());

        assert_eq!(
            entries,
            vec![
                "cached|/icons|example.png".to_string(),
                "stock|applications-internet".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ]
        );
    }

    #[test]
    fn resolves_stock_icons_from_icon_theme_roots() {
        let temp = std::env::temp_dir().join(format!(
            "nxc-stock-icon-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp);
        let icon_path = temp.join("hicolor/scalable/apps/example-stock.svg");
        fs::create_dir_all(icon_path.parent().unwrap()).unwrap();
        fs::write(&icon_path, "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>").unwrap();

        let index = build_stock_icon_index_from_roots(std::slice::from_ref(&temp));
        let resolved = index
            .get("example-stock")
            .and_then(|paths| paths.iter().find_map(|path| read_icon(path)));

        assert!(resolved.unwrap().starts_with("data:image/svg+xml;base64,"));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn parses_absolute_remote_icons() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/icon.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec!["remote|https://example.com/icon.png".to_string()])
        );
    }

    #[test]
    fn prefers_cached_icon_when_remote_comes_first() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/remote.png</icon>
              <icon type="cached">example.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec![
                "example.png".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ])
        );
    }

    #[test]
    fn prefers_cached_icon_when_remote_comes_after_cached() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="cached">example.png</icon>
              <icon type="remote">https://example.com/remote.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec![
                "example.png".to_string(),
                "remote|https://example.com/remote.png".to_string()
            ])
        );
    }

    #[test]
    fn falls_back_to_remote_when_cached_icon_is_missing() {
        let temp = std::env::temp_dir().join(format!(
            "nxc-appstream-fallback-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("64x64")).unwrap();

        let cached = format!("cached|{}|missing.png", temp.display());
        let remote = "remote|https://example.com/icon.png";
        assert!(
            load_icon(&cached).is_none(),
            "missing cached icon must fall back to the next source"
        );
        assert_eq!(load_icon(remote), Some("https://example.com/icon.png".to_string()));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn decodes_remote_icon_xml_entities() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/icon.png?a=1&amp;b=2</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(
            index.get("example"),
            Some(&vec![
                "remote|https://example.com/icon.png?a=1&b=2".to_string()
            ])
        );
    }

    #[test]
    fn ignores_relative_remote_icons() {
        let xml = r#"
            <component type="desktop-application">
              <pkgname>example</pkgname>
              <icon type="remote">icons/example.png</icon>
            </component>
        "#;

        assert!(parse_index(xml).is_empty());
    }

    #[test]
    fn resolves_unusual_cached_icon_size() {
        let temp = std::env::temp_dir().join(format!(
            "nxc-appstream-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("72x72")).unwrap();
        fs::write(temp.join("72x72").join("example.png"), b"png").unwrap();

        let icon = load_icon(&format!("{}|example.png", temp.display()));
        assert!(icon.is_some());

        let _ = fs::remove_dir_all(&temp);
    }
}
