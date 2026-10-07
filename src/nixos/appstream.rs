use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

pub fn icon_for_package(candidates: &[&str]) -> Option<String> {
    let index = INDEX.get_or_init(load_index);
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    for candidate in candidates {
        let icon_entry = index
            .get(*candidate)
            .or_else(|| index.get(&candidate.to_lowercase()));

        let Some(icon_entry) = icon_entry else {
            continue;
        };

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

    None
}

fn load_index() -> HashMap<String, String> {
    let mut index = HashMap::new();

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

        for (package, icon) in parse_index(&String::from_utf8_lossy(&output.stdout)) {
            index.entry(package).or_insert_with(|| format!("{}|{}", icon_root.display(), icon));
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
                    (xml.is_file() && icons.is_dir()).then_some((xml, icons))
                })
        })
        .collect()
}

fn parse_index(xml: &str) -> HashMap<String, String> {
    let mut index = HashMap::new();
    let mut offset = 0;

    while let Some(start) = xml[offset..].find("<component ") {
        let start = offset + start;
        let Some(end_rel) = xml[start..].find("</component>") else {
            break;
        };
        let end = start + end_rel + "</component>".len();
        let component = &xml[start..end];

        let icons = cached_icon_names(component);
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
            for icon in &icons {
                index.entry(package.clone()).or_insert_with(|| icon.clone());
                index.entry(package.to_lowercase()).or_insert_with(|| icon.clone());
            }
        }

        offset = end;
    }

    index
}

fn cached_icon_names(input: &str) -> Vec<String> {
    let mut icons = Vec::new();
    let mut offset = 0;

    while let Some(relative_start) = input[offset..].find("<icon") {
        let start = offset + relative_start;
        let Some(tag_end_rel) = input[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end_rel;
        let tag = &input[start..=tag_end];

        if (tag.starts_with("<icon ") || tag.starts_with("<icon>"))
            && element_attribute(tag, "icon", "type")
                .as_deref()
                .map_or(true, |value| value == "cached")
        {
            let content_start = tag_end + 1;
            if let Some(close_rel) = input[content_start..].find("</icon>") {
                let value = input[content_start..content_start + close_rel].trim();
                if !value.is_empty() {
                    icons.push(value.to_string());
                }
                offset = content_start + close_rel + "</icon>".len();
                continue;
            }
        }

        offset = tag_end + 1;
    }

    icons
}

fn element_text(input: &str, element: &str) -> Option<String> {
    element_texts(input, element).into_iter().next()
}

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

fn load_icon(entry: &str) -> Option<String> {
    let (root, icon_name) = entry.split_once('|')?;
    let root = Path::new(root);

    let mut names = vec![icon_name.to_string()];
    if let Some(name) = icon_name.strip_suffix(".desktop") {
        names.push(name.to_string());
    }

    let preferred_sizes = [
        "128x128@2",
        "128x128",
        "96x96@2",
        "96x96",
        "64x64@2",
        "64x64",
        "48x48@2",
        "48x48",
        "256x256",
        "scalable",
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
        assert_eq!(
            index.get("firefox"),
            Some(&"org.mozilla.firefox-64.png".to_string())
        );
        assert_eq!(
            index.get("org.mozilla.Firefox"),
            Some(&"org.mozilla.firefox-64.png".to_string())
        );
        assert_eq!(
            index.get("org.mozilla.firefox"),
            Some(&"org.mozilla.firefox-64.png".to_string())
        );
    }

    #[test]
    fn ignores_non_cached_icons() {
        let xml = r#"
            <component>
              <pkgname>example</pkgname>
              <icon type="remote">https://example.com/icon.png</icon>
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
