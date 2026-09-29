use super::flake;

// ============================================================
// Compatibility helpers
// ============================================================

pub fn add_package(package: &str) -> Result<String, String> {
    let content = flake::read_pkgs()?;
    add_package_to_content(&content, package)
}

pub fn remove_package(package: &str) -> Result<String, String> {
    let content = flake::read_pkgs()?;
    remove_package_from_content(&content, package)
}

pub fn enable_service(service: &str) -> Result<String, String> {
    let content = flake::read_core()?;
    add_service_to_content(&content, service, true)
}

pub fn disable_service(service: &str) -> Result<String, String> {
    let content = flake::read_core()?;
    add_service_to_content(&content, service, false)
}

// ============================================================
// Package content manipulation
// ============================================================

pub fn add_package_to_content(content: &str, package: &str) -> Result<String, String> {
    let marker = "environment.systemPackages";
    let marker_position = content
        .find(marker)
        .ok_or("environment.systemPackages section not found")?;

    let assignment = content[marker_position..]
        .find('=')
        .map(|offset| marker_position + offset)
        .ok_or("environment.systemPackages assignment not found")?;

    let list_start = content[assignment + 1..]
        .find('[')
        .map(|offset| assignment + 1 + offset)
        .ok_or("environment.systemPackages list not found")?;

    let list_end = find_matching_delimiter(content, list_start, '[', ']')
        .ok_or("environment.systemPackages list is not balanced")?;

    let indentation = indentation_for_list_item(content, list_start);
    let insertion = format!("\n{}{}", indentation, package);

    let mut result = content.to_string();
    result.insert_str(list_end, &insertion);
    Ok(result)
}

pub fn remove_package_from_content(content: &str, package: &str) -> Result<String, String> {
    let marker = "environment.systemPackages";
    let marker_position = content
        .find(marker)
        .ok_or("environment.systemPackages section not found")?;

    let assignment = content[marker_position..]
        .find('=')
        .map(|offset| marker_position + offset)
        .ok_or("environment.systemPackages assignment not found")?;

    let list_start = content[assignment + 1..]
        .find('[')
        .map(|offset| assignment + 1 + offset)
        .ok_or("environment.systemPackages list not found")?;

    let list_end = find_matching_delimiter(content, list_start, '[', ']')
        .ok_or("environment.systemPackages list is not balanced")?;

    let package_variants = [package.to_string(), format!("pkgs.{package}")];
    let region = &content[list_start + 1..list_end];

    let mut removed = false;
    let mut result = String::with_capacity(content.len());

    for line in region.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if package_variants.iter().any(|candidate| trimmed == candidate) {
            removed = true;
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    if !removed {
        return Err(format!("package '{}' not found as a standalone list item", package));
    }

    let mut output = String::with_capacity(content.len());
    output.push_str(&content[..list_start + 1]);

    if region.ends_with('\n') {
        output.push_str(&result);
    } else {
        output.push_str(result.trim_end_matches('\n'));
    }

    output.push_str(&content[list_end..]);
    Ok(output)
}

// ============================================================
// Service content manipulation
// ============================================================

pub fn add_service_to_content(
    content: &str,
    service: &str,
    enabled: bool,
) -> Result<String, String> {
    let desired = format!("{};", enabled);
    let prefixes = [
        format!("systemd.services.{service}.enable"),
        format!("services.{service}.enable"),
    ];

    for prefix in &prefixes {
        if let Some((start, end)) = find_assignment_line(content, prefix) {
            let mut result = content.to_string();
            result.replace_range(start..end, &desired);
            return Ok(result);
        }
    }

    let closing_brace = content
        .rfind('}')
        .ok_or("Nix module closing brace not found")?;

    let setting = format!("  {}.enable = {};\n", prefixes[1].trim_end_matches(".enable"), enabled);

    let mut result = content.to_string();
    result.insert_str(closing_brace, &setting);
    Ok(result)
}

fn find_assignment_line(content: &str, attribute: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;

    for line in content.split_inclusive('\n') {
        let code = line.split_once('#').map_or(line, |(code, _)| code);
        if let Some(attribute_pos) = code.find(attribute) {
            let after = &code[attribute_pos + attribute.len()..];
            if after.trim_start().starts_with('=') {
                let value_start = code.len() - after.len();
                let equals = after.find('=')? + value_start;
                let value = code[equals + 1..].trim();
                let value_end = equals + 1 + code[equals + 1..].find(value)?;
                let absolute_start = offset + equals + 1 + code[equals + 1..].find(value)?;
                return Some((absolute_start, absolute_start + value.len()));
            }
        }
        offset += line.len();
    }

    None
}

fn indentation_for_list_item(content: &str, list_start: usize) -> String {
    let after = &content[list_start + 1..];
    after
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line[..line.len() - line.trim_start().len()].to_string())
        .unwrap_or_else(|| "  ".to_string())
}

fn find_matching_delimiter(
    content: &str,
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;

    for (offset, ch) in content[start..].char_indices() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '#' => in_comment = true,
            '"' => in_string = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn adds_package_to_a_nonstandard_system_package_expression() {
        let content = r#"{
  environment.systemPackages = lib.mkAfter [
    pkgs.git
  ];
}
"#;

        let result = add_package_to_content(content, "firefox").unwrap();

        assert!(result.contains("pkgs.git"));
        assert!(result.contains("firefox"));
    }

    #[test]
    fn removes_package_as_a_standalone_item() {
        let content = r#"{
  environment.systemPackages = [
    pkgs.firefox
    git
  ];
}
"#;

        let result = remove_package_from_content(content, "firefox").unwrap();

        assert!(!result.contains("pkgs.firefox"));
        assert!(result.contains("git"));
    }

    #[test]
    fn updates_existing_service_option() {
        let content = r#"{
  services.openssh.enable = true;
}
"#;

        let result = add_service_to_content(content, "openssh", false).unwrap();

        assert!(result.contains("services.openssh.enable = false;"));
        assert!(!result.contains("services.openssh.enable = true;"));
    }

    #[test]
    fn updates_systemd_service_option() {
        let content = r#"{
  systemd.services.example.enable = false;
}
"#;

        let result = add_service_to_content(content, "example", true).unwrap();

        assert!(result.contains("systemd.services.example.enable = true;"));
    }

    #[test]
    fn adds_service_to_existing_module() {
        let content = "{\n}\n";

        let result = add_service_to_content(content, "openssh", true).unwrap();

        assert!(result.contains("services.openssh.enable = true;"));
    }

    #[test]
    fn preserves_unrelated_content() {
        let directory = std::env::temp_dir().join(format!("nxc-module-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();

        let path = directory.join("config.nix");
        fs::write(&path, "{ services.openssh.enable = true; }").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("services.openssh.enable = true;"));

        let _ = fs::remove_dir_all(directory);
    }
}
