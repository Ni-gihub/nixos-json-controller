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

    let region = &content[list_start + 1..list_end];
    if simple_package_list_contains(region, package) {
        return Err(format!("package '{}' is already present in the package list", package));
    }

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

    let region = &content[list_start + 1..list_end];

    if is_simple_package_list(region) {
        if let Some((start, end)) = find_package_token(region, package) {
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..list_start + 1]);
            result.push_str(&region[..start]);
            result.push_str(&region[end..]);
            result.push_str(&content[list_end..]);
            return Ok(result);
        }

        return Err(format!(
            "package '{}' not found as a standalone list item",
            package
        ));
    }

    // Keep the conservative line-based fallback for lists containing more
    // complex expressions. It avoids rewriting a package token inside an
    // arbitrary Nix expression.
    let variants = [package.to_string(), format!("pkgs.{package}")];
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
        return Err(format!(
            "package '{}' not found as a standalone list item",
            package
        ));
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

fn is_simple_package_list(region: &str) -> bool {
    !region.chars().any(|c| matches!(c, '(' | ')' | '{' | '}' | ';'))
        && !region.contains(" if ")
        && !region.contains(" then ")
        && !region.contains(" else ")
        && !region.contains(" with ")
}

fn simple_package_list_contains(region: &str, package: &str) -> bool {
    find_package_token(region, package).is_some()
}

fn find_package_token(region: &str, package: &str) -> Option<(usize, usize)> {
    let variants = [package.to_string(), format!("pkgs.{package}")];

    for variant in variants {
        let mut search_start = 0usize;

        while let Some(relative) = region[search_start..].find(&variant) {
            let start = search_start + relative;
            let end = start + variant.len();

            let before = region[..start].chars().next_back();
            let after = region[end..].chars().next();

            let valid_before = before.map_or(true, |c| c.is_whitespace());
            let valid_after = after.map_or(true, |c| c.is_whitespace());

            if valid_before && valid_after {
                return Some((start, end));
            }

            search_start = end;
        }
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

pub(crate) fn find_matching_delimiter(
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
        let content =
            "{\n  environment.systemPackages = [\n    pkgs.firefox\n    git\n  ];\n}\n";
        let result = remove_package_from_content(content, "firefox").unwrap();

        assert!(!result.contains("pkgs.firefox"));
        assert!(result.contains("git"));
    }

    #[test]
    fn removes_package_from_inline_simple_list() {
        let content = "{ environment.systemPackages = [ pkgs.firefox git ]; }";
        let result = remove_package_from_content(content, "firefox").unwrap();

        assert_eq!(
            result,
            "{ environment.systemPackages = [  git ]; }"
        );
        assert!(result.contains("git"));
        assert!(!result.contains("firefox"));
    }

    #[test]
    fn rejects_duplicate_package_in_simple_list() {
        let content = "{ environment.systemPackages = [ pkgs.firefox git ]; }";
        let result = add_package_to_content(content, "firefox");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already present"));
    }

    #[test]
    fn does_not_remove_package_substring_from_other_identifier() {
        let content = "{ environment.systemPackages = [ pkgs.firefox-nightly ]; }";
        let result = remove_package_from_content(content, "firefox");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
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
        let content =
            "{\n  services.openssh.enable = true;\n  services.xserver.enable = false;\n}\n";
        let result = add_service_to_content(content, "openssh", false).unwrap();

        assert!(result.contains("services.xserver.enable = false;"));
        assert!(result.contains("services.openssh.enable = false;"));
    }
}
