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

pub fn add_package_to_content(content: &str, package: &str) -> Result<String, String> {
    let marker = "environment.systemPackages";
    let marker_position = content.find(marker).ok_or("environment.systemPackages section not found")?;

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
    let insertion = if list_start + 1 == list_end {
        format!("\n{}{}", indentation, package)
    } else {
        format!("\n{}{}", indentation, package)
    };

    let mut result = content.to_string();
    result.insert_str(list_end, &insertion);
    Ok(result)
}

pub fn remove_package_from_content(content: &str, package: &str) -> Result<String, String> {
    let marker = "environment.systemPackages";
    let marker_position = content.find(marker).ok_or("environment.systemPackages section not found")?;

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

    let variants = [package.to_string(), format!("pkgs.{package}")];
    let region = &content[list_start + 1..list_end];

    let mut removed = false;
    let mut output_region = String::with_capacity(region.len());

    for line in region.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if variants.iter().any(|candidate| trimmed == candidate) {
            removed = true;
            continue;
        }

        output_region.push_str(line);
        output_region.push('\n');
    }

    if !removed {
        return Err(format!("package '{}' not found as a standalone list item", package));
    }

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..list_start + 1]);

    if region.ends_with('\n') {
        result.push_str(&output_region);
    } else {
        result.push_str(output_region.trim_end_matches('\n'));
    }

    result.push_str(&content[list_end..]);
    Ok(result)
}

pub fn add_service_to_content(
    content: &str,
    service: &str,
    enabled: bool,
) -> Result<String, String> {
    let attributes = [
        format!("systemd.services.{service}.enable"),
        format!("services.{service}.enable"),
    ];

    for attribute in &attributes {
        if let Some((start, end)) = find_boolean_assignment(content, attribute) {
            let mut result = content.to_string();
            result.replace_range(start..end, if enabled { "true" } else { "false" });
            return Ok(result);
        }
    }

    let closing_brace = content
        .rfind('}')
        .ok_or("Nix module closing brace not found")?;

    let setting = format!("  services.{service}.enable = {enabled};\n");
    let mut result = content.to_string();
    result.insert_str(closing_brace, &setting);
    Ok(result)
}

fn find_boolean_assignment(content: &str, attribute: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;

    for line in content.split_inclusive('\n') {
        let code = line.split_once('#').map_or(line, |(code, _)| code);

        if let Some(attribute_pos) = code.find(attribute) {
            let after_attribute = &code[attribute_pos + attribute.len()..];
            let equals = after_attribute.find('=')?;
            let value_part = &after_attribute[equals + 1..];
            let value_start = value_part.len() - value_part.trim_start().len();
            let value = value_part.trim_start();

            if value.starts_with("true") {
                let start = offset + attribute_pos + attribute.len() + equals + 1 + value_start;
                return Some((start, start + 4));
            }

            if value.starts_with("false") {
                let start = offset + attribute_pos + attribute.len() + equals + 1 + value_start;
                return Some((start, start + 5));
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

    #[test]
    fn adds_package_to_a_nonstandard_system_package_expression() {
        let content = "{\n  environment.systemPackages = lib.mkAfter [\n    pkgs.git\n  ];\n}\n";
        let result = add_package_to_content(content, "firefox").unwrap();
        assert!(result.contains("pkgs.git"));
        assert!(result.contains("firefox"));
    }

    #[test]
    fn removes_package_as_a_standalone_item() {
        let content = "{\n  environment.systemPackages = [\n    pkgs.firefox\n    git\n  ];\n}\n";
        let result = remove_package_from_content(content, "firefox").unwrap();
        assert!(!result.contains("pkgs.firefox"));
        assert!(result.contains("git"));
    }

    #[test]
    fn updates_existing_service_option() {
        let content = "{\n  services.openssh.enable = true;\n}\n";
        let result = add_service_to_content(content, "openssh", false).unwrap();
        assert!(result.contains("services.openssh.enable = false;"));
        assert!(!result.contains("services.openssh.enable = true;"));
    }

    #[test]
    fn updates_systemd_service_option() {
        let content = "{\n  systemd.services.example.enable = false;\n}\n";
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
    fn does_not_modify_unrelated_boolean_values() {
        let content = "{\n  services.openssh.enable = true;\n  services.xserver.enable = false;\n}\n";
        let result = add_service_to_content(content, "openssh", false).unwrap();
        assert!(result.contains("services.xserver.enable = false;"));
        assert!(result.contains("services.openssh.enable = false;"));
    }
}
