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
        let Some(icon_name) = index.get(*candidate) else {
            continue;
        };

        if let Ok(icons) = cache.lock() {
            if let Some(cached) = icons.get(icon_name) {
                return cached.clone();
            }
        }

        let icon = load_icon(icon_name);
        if let Ok(mut icons) = cache.lock() {
            icons.insert(icon_name.clone(), icon.clone());
        }
        if icon.is_some() {
            return icon;
        }
    }

    None
}

fn load_index() -> HashMap<String, String> {
    let Some(xml_path) = find_xml() else {
        return HashMap::new();
    };

    let Ok(output) = Command::new("gzip")
        .args(["-dc"])
        .arg(xml_path)
        .output()
    else {
        return HashMap::new();
    };

    if !output.status.success() {
        return HashMap::new();
    }

    parse_index(&String::from_utf8_lossy(&output.stdout))
}

fn find_xml() -> Option<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(root) = env::var("NXC_APPSTREAM_DATA") {
        roots.push(PathBuf::from(root));
    }

    roots.extend([
        PathBuf::from("/run/current-system/sw"),
        PathBuf::from("/usr"),
    ]);

    roots.into_iter().find_map(|root| {
        let candidates = [
            root.join("share/swcatalog/xml/nixos-unstable.xml.gz"),
            root.join("share/app-info/xmls/nixos_x86_64_linux.yml.gz"),
        ];
        candidates.into_iter().find(|path| path.is_file())
    })
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

        let Some(icon) = element_text(component, "icon") else {
            offset = end;
            continue;
        };

        let icon_type = element_attribute(component, "icon", "type");
        if icon_type.as_deref().is_some_and(|value| value != "cached") {
            offset = end;
            continue;
        }

        if let Some(pkgname) = element_text(component, "pkgname") {
            for package in pkgname.split_whitespace() {
                index.entry(package.to_string()).or_insert_with(|| icon.clone());
            }
        }

        if let Some(id) = element_text(component, "id") {
            index.entry(id).or_insert_with(|| icon.clone());
        }

        offset = end;
    }

    index
}

fn element_text(input: &str, element: &str) -> Option<String> {
    let open = format!("<{element}");
    let start = input.find(&open)?;
    let content_start = input[start..].find('>')? + start + 1;
    let close = format!("</{element}>");
    let content_end = input[content_start..].find(&close)? + content_start;
    let value = input[content_start..content_end].trim();
    (!value.is_empty()).then(|| value.to_string())
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

fn load_icon(icon_name: &str) -> Option<String> {
    let root = appstream_root()?;

    for size in ["128x128", "64x64", "48x48"] {
        let path = root
            .join("share/swcatalog/icons/nixos-unstable")
            .join(size)
            .join(icon_name);

        if let Some(data_url) = read_icon(&path) {
            return Some(data_url);
        }
    }

    None
}

fn appstream_root() -> Option<PathBuf> {
    if let Ok(root) = env::var("NXC_APPSTREAM_DATA") {
        let path = PathBuf::from(root);
        if path.is_dir() {
            return Some(path);
        }
    }

    for root in [Path::new("/run/current-system/sw"), Path::new("/usr")] {
        if root.join("share/swcatalog").is_dir() {
            return Some(root.to_path_buf());
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
    fn parses_cached_icon_for_package() {
        let xml = r#"
            <component type="desktop-application">
              <id>org.mozilla.firefox</id>
              <pkgname>firefox</pkgname>
              <icon type="cached">org.mozilla.firefox.png</icon>
            </component>
        "#;

        let index = parse_index(xml);
        assert_eq!(index.get("firefox"), Some(&"org.mozilla.firefox.png".to_string()));
        assert_eq!(
            index.get("org.mozilla.firefox"),
            Some(&"org.mozilla.firefox.png".to_string())
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
}
