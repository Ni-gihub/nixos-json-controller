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
    let (list_start, list_end) = find_editable_system_packages_list(content)?;
    let code = nix_code_mask(content);
    let items = parse_simple_package_list(content, list_start + 1, list_end, &code)?;
    let package_name = normalize_package_name(package).ok_or_else(|| {
        format!(
            "package '{}' is not a simple Nix package identifier; refusing to edit",
            package
        )
    })?;

    if items.iter().any(|item| item.name == package_name) {
        return Err(format!(
            "package '{}' is already present in the package list",
            package
        ));
    }

    let indentation = indentation_for_list_item(content, list_start);
    let line_start = content[..list_end]
        .rfind('\n')
        .map(|offset| offset + 1)
        .unwrap_or(0);
    let closing_prefix = &content[line_start..list_end];

    let (insert_at, insertion) = if closing_prefix.trim().is_empty() {
        (line_start, format!("{}{}\n", indentation, package))
    } else {
        let separator = content[..list_end]
            .chars()
            .next_back()
            .map_or(true, |ch| !ch.is_whitespace());
        (
            list_end,
            format!("{}{} ", if separator { " " } else { "" }, package),
        )
    };

    let mut result = content.to_string();
    result.insert_str(insert_at, &insertion);
    Ok(result)
}

pub fn remove_package_from_content(content: &str, package: &str) -> Result<String, String> {
    let (list_start, list_end) = find_editable_system_packages_list(content)?;
    let code = nix_code_mask(content);
    let items = parse_simple_package_list(content, list_start + 1, list_end, &code)?;
    let package_name = normalize_package_name(package).ok_or_else(|| {
        format!(
            "package '{}' is not a simple Nix package identifier; refusing to edit",
            package
        )
    })?;

    let matches: Vec<&PackageAtom> = items
        .iter()
        .filter(|item| item.name == package_name)
        .collect();

    match matches.as_slice() {
        [] => Err(format!(
            "package '{}' not found as a standalone list item",
            package
        )),
        [item] => {
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..item.start]);
            result.push_str(&content[item.end..]);
            Ok(result)
        }
        _ => Err(format!(
            "package '{}' occurs more than once in the editable list; refusing to guess which entry to remove",
            package
        )),
    }
}

#[derive(Debug)]
struct PackageAtom {
    start: usize,
    end: usize,
    name: String,
}

fn find_editable_system_packages_list(content: &str) -> Result<(usize, usize), String> {
    let marker = "environment.systemPackages";
    let code = nix_code_mask(content);
    let mut assignments = Vec::new();
    let mut search_from = 0;

    while let Some(relative) = content[search_from..].find(marker) {
        let start = search_from + relative;
        let end = start + marker.len();
        search_from = end;

        if !code_range_is_active(&code, start, end) {
            continue;
        }

        let before = content[..start].chars().next_back();
        if before.map_or(false, |ch| is_nix_identifier_char(ch) || ch == '.') {
            continue;
        }

        let after = content[end..].chars().next();
        if after.map_or(false, |ch| is_nix_identifier_char(ch) || ch == '.') {
            continue;
        }

        let equals = skip_nix_trivia(content, end, content.len());
        if equals >= content.len() || content.as_bytes()[equals] != b'=' || !code[equals] {
            continue;
        }

        let next = content.as_bytes().get(equals + 1).copied();
        if matches!(next, Some(b'=' | b'>')) {
            continue;
        }

        assignments.push(equals);
    }

    if assignments.is_empty() {
        return Err("environment.systemPackages assignment not found".to_string());
    }

    if assignments.len() != 1 {
        return Err(format!(
            "found {} environment.systemPackages assignments; refusing to choose one",
            assignments.len()
        ));
    }

    let mut cursor = skip_nix_trivia(content, assignments[0] + 1, content.len());
    let mut consumed_with = false;
    let mut consumed_wrapper = false;

    loop {
        if !consumed_with && active_token_at(content, &code, cursor, "with") {
            let pkgs = skip_nix_trivia(content, cursor + "with".len(), content.len());
            if !active_token_at(content, &code, pkgs, "pkgs") {
                return Err(
                    "environment.systemPackages uses an unsupported 'with' expression; refusing to edit"
                        .to_string(),
                );
            }

            let semicolon = skip_nix_trivia(content, pkgs + "pkgs".len(), content.len());
            if semicolon >= content.len()
                || content.as_bytes()[semicolon] != b';'
                || !code[semicolon]
            {
                return Err(
                    "environment.systemPackages has an unsupported 'with' expression; refusing to edit"
                        .to_string(),
                );
            }

            cursor = skip_nix_trivia(content, semicolon + 1, content.len());
            consumed_with = true;
            continue;
        }

        // mkAfter is a supported wrapper because its sole list argument remains
        // the exact list being edited. Conditional and combining functions are
        // deliberately not accepted here.
        if !consumed_wrapper && active_token_at(content, &code, cursor, "lib.mkAfter") {
            cursor = skip_nix_trivia(content, cursor + "lib.mkAfter".len(), content.len());
            consumed_wrapper = true;
            continue;
        }

        break;
    }

    if cursor >= content.len() || content.as_bytes()[cursor] != b'[' || !code[cursor] {
        return Err(
            "environment.systemPackages is not a directly editable list (for example, it may use a conditional or combined expression); no changes were made"
                .to_string(),
        );
    }

    let list_end = find_matching_delimiter(content, cursor, '[', ']').ok_or_else(|| {
        "environment.systemPackages list is not balanced; no changes were made".to_string()
    })?;

    let semicolon = skip_nix_trivia(content, list_end + 1, content.len());
    if semicolon >= content.len() || content.as_bytes()[semicolon] != b';' || !code[semicolon] {
        return Err(
            "environment.systemPackages list is part of a larger expression; refusing to edit it"
                .to_string(),
        );
    }

    Ok((cursor, list_end))
}

fn parse_simple_package_list(
    content: &str,
    start: usize,
    end: usize,
    code: &[bool],
) -> Result<Vec<PackageAtom>, String> {
    let mut items = Vec::new();
    let mut cursor = start;

    loop {
        cursor = skip_nix_trivia(content, cursor, end);
        if cursor >= end {
            return Ok(items);
        }

        if !code[cursor] {
            return Err(
                "package list contains a string or unsupported Nix expression; refusing to edit"
                    .to_string(),
            );
        }

        let item_start = cursor;
        let bytes = content.as_bytes();

        if content[cursor..end].starts_with("pkgs.")
            && code_range_is_active(code, cursor, cursor + "pkgs.".len())
        {
            cursor += "pkgs.".len();
            let name_end = read_nix_identifier_end(content, cursor, end);
            if name_end == cursor {
                return Err(
                    "package list contains an incomplete pkgs attribute; refusing to edit"
                        .to_string(),
                );
            }
            cursor = name_end;
        } else {
            cursor = read_nix_identifier_end(content, cursor, end);
            if cursor == item_start {
                return Err(
                    "package list contains a non-package expression; refusing to edit".to_string(),
                );
            }
        }

        let token = &content[item_start..cursor];
        let name = token.strip_prefix("pkgs.").unwrap_or(token);
        if !is_valid_package_identifier(name) || is_nix_keyword(name) {
            return Err(
                "package list contains a non-package expression; refusing to edit".to_string(),
            );
        }

        if !code_range_is_active(code, item_start, cursor) {
            return Err(
                "package list contains a token inside a comment or string; refusing to edit"
                    .to_string(),
            );
        }

        if cursor < end
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'#'
            && !content[cursor..end].starts_with("/*")
        {
            return Err(
                "package list contains a non-package expression; refusing to edit".to_string(),
            );
        }

        items.push(PackageAtom {
            start: item_start,
            end: cursor,
            name: name.to_string(),
        });
    }
}

fn normalize_package_name(package: &str) -> Option<&str> {
    let name = package.strip_prefix("pkgs.").unwrap_or(package);
    is_valid_package_identifier(name).then_some(name)
}

fn is_valid_package_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '\''))
}

fn read_nix_identifier_end(content: &str, start: usize, limit: usize) -> usize {
    if start >= limit {
        return start;
    }

    let bytes = content.as_bytes();
    if !(bytes[start].is_ascii_alphabetic() || bytes[start] == b'_') {
        return start;
    }

    let mut end = start + 1;
    while end < limit
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'-' | '\''))
    {
        end += 1;
    }
    end
}

fn is_nix_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '\'')
}

fn is_nix_keyword(name: &str) -> bool {
    matches!(
        name,
        "assert" | "else" | "if" | "in" | "inherit" | "let" | "or" | "rec" | "then" | "with"
    )
}

fn active_token_at(content: &str, code: &[bool], start: usize, token: &str) -> bool {
    let end = start.saturating_add(token.len());
    if end > content.len()
        || !content[start..].starts_with(token)
        || !code_range_is_active(code, start, end)
    {
        return false;
    }

    let before = content[..start].chars().next_back();
    if before.map_or(false, |ch| is_nix_identifier_char(ch) || ch == '.') {
        return false;
    }

    let after = content[end..].chars().next();
    !after.map_or(false, |ch| is_nix_identifier_char(ch) || ch == '.')
}

fn skip_nix_trivia(content: &str, mut cursor: usize, limit: usize) -> usize {
    let limit = limit.min(content.len());
    let bytes = content.as_bytes();

    loop {
        while cursor < limit && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        if cursor >= limit {
            return cursor;
        }

        if bytes[cursor] == b'#' {
            while cursor < limit && bytes[cursor] != b'\n' {
                cursor += 1;
            }
            continue;
        }

        if cursor + 1 < limit && bytes[cursor] == b'/' && bytes[cursor + 1] == b'*' {
            cursor += 2;
            let mut depth = 1usize;
            while cursor < limit && depth > 0 {
                if cursor + 1 < limit && bytes[cursor] == b'/' && bytes[cursor + 1] == b'*' {
                    depth += 1;
                    cursor += 2;
                } else if cursor + 1 < limit && bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
                    depth -= 1;
                    cursor += 2;
                } else {
                    cursor += 1;
                }
            }
            continue;
        }

        return cursor;
    }
}

fn code_range_is_active(code: &[bool], start: usize, end: usize) -> bool {
    start < end && end <= code.len() && code[start..end].iter().all(|active| *active)
}

fn nix_code_mask(content: &str) -> Vec<bool> {
    // The mask keeps byte offsets aligned with the source, which lets the
    // editor replace ranges without changing comments or string contents.
    const CODE: u8 = 0;
    const LINE_COMMENT: u8 = 1;
    const DOUBLE_STRING: u8 = 2;
    const INDENTED_STRING: u8 = 3;
    const BLOCK_COMMENT: u8 = 4;

    let bytes = content.as_bytes();
    let mut active = vec![false; bytes.len()];
    let mut state = CODE;
    let mut escaped = false;
    let mut block_depth = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        match state {
            CODE => {
                if bytes[cursor] == b'#' {
                    state = LINE_COMMENT;
                    cursor += 1;
                } else if bytes[cursor] == b'"' {
                    state = DOUBLE_STRING;
                    escaped = false;
                    cursor += 1;
                } else if cursor + 1 < bytes.len()
                    && bytes[cursor] == b'\''
                    && bytes[cursor + 1] == b'\''
                {
                    state = INDENTED_STRING;
                    cursor += 2;
                } else if cursor + 1 < bytes.len()
                    && bytes[cursor] == b'/'
                    && bytes[cursor + 1] == b'*'
                {
                    state = BLOCK_COMMENT;
                    block_depth = 1;
                    cursor += 2;
                } else {
                    active[cursor] = true;
                    cursor += 1;
                }
            }
            LINE_COMMENT => {
                if bytes[cursor] == b'\n' {
                    active[cursor] = true;
                    state = CODE;
                }
                cursor += 1;
            }
            DOUBLE_STRING => {
                if escaped {
                    escaped = false;
                    cursor += 1;
                } else if bytes[cursor] == b'\\' {
                    escaped = true;
                    cursor += 1;
                } else if bytes[cursor] == b'"' {
                    state = CODE;
                    cursor += 1;
                } else {
                    cursor += 1;
                }
            }
            INDENTED_STRING => {
                if cursor + 1 < bytes.len() && bytes[cursor] == b'\'' && bytes[cursor + 1] == b'\'' {
                    let third = bytes.get(cursor + 2).copied();
                    if third == Some(b'$') && bytes.get(cursor + 3) == Some(&b'{') {
                        cursor += 4;
                    } else if matches!(third, Some(b'\\' | b'\'' | b'$')) {
                        cursor += 3;
                    } else {
                        state = CODE;
                        cursor += 2;
                    }
                } else {
                    cursor += 1;
                }
            }
            BLOCK_COMMENT => {
                if cursor + 1 < bytes.len() && bytes[cursor] == b'/' && bytes[cursor + 1] == b'*' {
                    block_depth += 1;
                    cursor += 2;
                } else if cursor + 1 < bytes.len()
                    && bytes[cursor] == b'*'
                    && bytes[cursor + 1] == b'/'
                {
                    block_depth -= 1;
                    cursor += 2;
                    if block_depth == 0 {
                        state = CODE;
                    }
                } else {
                    cursor += 1;
                }
            }
            _ => unreachable!("unknown lexer state"),
        }
    }

    active
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
            let start = offset + attribute_pos + attribute.len() + equals + 1 + value_start;

            if value.starts_with("true") {
                return Some((start, start + 4));
            }

            if value.starts_with("false") {
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

pub(crate) fn find_matching_delimiter(
    content: &str,
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let code = nix_code_mask(content);
    if !content[start..].starts_with(open) || !code.get(start).copied().unwrap_or(false) {
        return None;
    }

    let mut depth = 0usize;
    for (offset, ch) in content[start..].char_indices() {
        let absolute = start + offset;
        if !code.get(absolute).copied().unwrap_or(false) {
            continue;
        }

        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(absolute);
            }
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
    fn removes_package_from_inline_simple_list() {
        let content = "{ environment.systemPackages = [ pkgs.firefox git ]; }";
        let result = remove_package_from_content(content, "firefox").unwrap();

        assert_eq!(result, "{ environment.systemPackages = [  git ]; }");
        assert!(result.contains("git"));
        assert!(!result.contains("firefox"));
    }

    #[test]
    fn adds_package_to_an_inline_simple_list() {
        let content = "{ environment.systemPackages = [ pkgs.git ]; }";
        let result = add_package_to_content(content, "firefox").unwrap();

        assert_eq!(
            result,
            "{ environment.systemPackages = [ pkgs.git firefox ]; }"
        );
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
    fn ignores_assignment_like_text_in_comments_and_strings() {
        let content = r#"
# environment.systemPackages = [ pkgs.commentOnly ];
{
  description = "environment.systemPackages = [ pkgs.stringOnly ]";
  environment.systemPackages = [
    pkgs.git # a fake closing bracket ] must be ignored
    pkgs.curl
  ];
}
"#;
        let result = add_package_to_content(content, "firefox").unwrap();

        assert!(result.contains("# environment.systemPackages = [ pkgs.commentOnly ];"));
        assert!(result.contains("\"environment.systemPackages = [ pkgs.stringOnly ]\""));
        assert!(result.contains("pkgs.git # a fake closing bracket ] must be ignored"));
        assert!(result.contains("    firefox"));
    }

    #[test]
    fn ignores_attribute_markers_inside_indented_strings() {
        let content = r#"{
  description = ''
    environment.systemPackages = [ notADeclaration ];
  '';
  environment.systemPackages = [ pkgs.git ];
}
"#;
        let result = add_package_to_content(content, "firefox").unwrap();

        assert!(result.contains("environment.systemPackages = [ notADeclaration ];"));
        assert!(result.contains("[ pkgs.git firefox ]"));
    }

    #[test]
    fn refuses_to_edit_conditional_and_concatenated_expressions() {
        let content = "{\n  environment.systemPackages = lib.optionals condition [ pkgs.git ] ++ [ pkgs.curl ];\n}\n";
        let result = add_package_to_content(content, "firefox");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directly editable list"));
    }

    #[test]
    fn refuses_to_treat_a_merge_expression_list_as_the_option_list() {
        let content =
            "{\n  environment.systemPackages = lib.mkMerge [ [ pkgs.git ] [ pkgs.curl ] ];\n}\n";
        let result = remove_package_from_content(content, "git");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directly editable list"));
    }

    #[test]
    fn refuses_to_remove_from_a_composite_expression_without_mutating_content() {
        let content = "{\n  environment.systemPackages = lib.optionals condition [ pkgs.firefox ] ++ [ pkgs.curl ];\n}\n";
        let result = remove_package_from_content(content, "firefox");

        assert!(result.is_err());
        assert_eq!(
            content,
            "{\n  environment.systemPackages = lib.optionals condition [ pkgs.firefox ] ++ [ pkgs.curl ];\n}\n"
        );
    }

    #[test]
    fn ignores_a_package_mentioned_only_in_a_comment_for_duplicate_detection() {
        let content = "{\n  environment.systemPackages = [\n    # firefox was considered here\n    pkgs.git\n  ];\n}\n";
        let result = add_package_to_content(content, "firefox").unwrap();

        assert!(result.contains("# firefox was considered here"));
        assert!(result.contains("    firefox"));
    }

    #[test]
    fn refuses_to_guess_between_multiple_package_assignments() {
        let content = "{\n  environment.systemPackages = [ pkgs.git ];\n  environment.systemPackages = [ pkgs.curl ];\n}\n";
        let result = add_package_to_content(content, "firefox");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("assignments"));
    }

    #[test]
    fn refuses_to_edit_a_list_that_contains_non_package_expressions() {
        let content = "{\n  environment.systemPackages = [ pkgs.git \"literal [ and ]\" ];\n}\n";
        let result = add_package_to_content(content, "firefox");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("string or unsupported"));
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
