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

/// Read the core module and safely enable the named service.
pub fn enable_service(service: &str) -> Result<String, String> {
    validate_service_name(service)?;
    let content = flake::read_core()?;
    add_service_to_content(&content, service, true)
}

/// Read the core module and safely disable the named service.
pub fn disable_service(service: &str) -> Result<String, String> {
    validate_service_name(service)?;
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
            let name_end = read_nix_attrpath_end(content, cursor, end);
            if name_end == cursor {
                return Err(
                    "package list contains an incomplete pkgs attribute; refusing to edit"
                        .to_string(),
                );
            }
            cursor = name_end;
        } else {
            cursor = read_nix_attrpath_end(content, cursor, end);
            if cursor == item_start {
                return Err(
                    "package list contains a non-package expression; refusing to edit".to_string(),
                );
            }
        }

        let token = &content[item_start..cursor];
        let name = token.strip_prefix("pkgs.").unwrap_or(token);
        if !is_valid_package_identifier(name) || name.split('.').any(is_nix_keyword) {
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
    !name.is_empty()
        && name.split('.').all(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return false;
            };

            (first.is_ascii_alphabetic() || first == '_')
                && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '\''))
        })
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
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'-' | b'\''))
    {
        end += 1;
    }
    end
}

fn read_nix_attrpath_end(content: &str, start: usize, limit: usize) -> usize {
    let mut end = read_nix_identifier_end(content, start, limit);
    if end == start {
        return start;
    }

    let bytes = content.as_bytes();
    while end < limit && bytes[end] == b'.' {
        let next_start = end + 1;
        let next_end = read_nix_identifier_end(content, next_start, limit);
        if next_end == next_start {
            break;
        }
        end = next_end;
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

pub(crate) fn nix_code_mask(content: &str) -> Vec<bool> {
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
                if cursor + 1 < bytes.len() && bytes[cursor] == b'\'' && bytes[cursor + 1] == b'\''
                {
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

/// Validate a service name before it is interpolated into a Nix attribute path.
fn validate_service_name(service: &str) -> Result<(), String> {
    let mut chars = service.chars();
    let valid = chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(is_nix_identifier_char)
        && !is_nix_keyword(service);

    if !valid {
        return Err(format!(
            "service '{}' is not a simple Nix attribute name; refusing to edit",
            service
        ));
    }

    Ok(())
}

/// Add or update a service enable option in the module's returned attribute set.
///
/// Only simple Nix attribute names are accepted. Nested attribute sets are not
/// treated as direct NixOS options, and unsupported existing values are rejected.
/// Find a safely editable service declaration, preferring the high-level NixOS option.
fn existing_service_assignment(
    content: &str,
    service: &str,
) -> Result<Option<(String, usize, usize)>, String> {
    validate_service_name(service)?;
    let attributes = [
        format!("services.{service}.enable"),
        format!("systemd.services.{service}.enable"),
    ];

    for attribute in &attributes {
        if let Some((start, end)) = find_boolean_assignment(content, attribute)? {
            return Ok(Some((attribute.clone(), start, end)));
        }
    }

    Ok(None)
}

/// Return the service option that a write to this content will target.
pub fn service_option_for_content(content: &str, service: &str) -> Result<String, String> {
    Ok(existing_service_assignment(content, service)?
        .map(|(option, _, _)| option)
        .unwrap_or_else(|| format!("services.{service}.enable")))
}

/// Add or update a service enable option in the module's returned attribute set.
///
/// Only simple Nix attribute names are accepted. Nested attribute sets are not
/// treated as direct NixOS options, and unsupported existing values are rejected.
pub fn add_service_to_content(
    content: &str,
    service: &str,
    enabled: bool,
) -> Result<String, String> {
    match existing_service_assignment(content, service)? {
        Some((_, start, end)) => {
            let mut result = content.to_string();
            result.replace_range(start..end, if enabled { "true" } else { "false" });
            Ok(result)
        }
        None => {
            let code = nix_code_mask(content);
            let (_, closing_brace) = returned_module_attrset(content, &code)?;
            let newline = if content[..closing_brace].ends_with('\n') {
                ""
            } else {
                "\n"
            };
            let setting = format!("{newline}  services.{service}.enable = {enabled};\n");
            let mut result = content.to_string();
            result.insert_str(closing_brace, &setting);
            Ok(result)
        }
    }
}

/// Locate the attribute set returned by a NixOS module function.
///
/// Supports plain attribute sets, `{ ... }: { ... }`, `args@{ ... }: { ... }`,
/// `{ ... }@args: { ... }`, and `let ... in { ... }`.
/// Attribute sets used by let bindings are skipped rather than mistaken for the
/// module's returned value.
fn returned_module_attrset(content: &str, code: &[bool]) -> Result<(usize, usize), String> {
    let bytes = content.as_bytes();
    let first_token = skip_nix_trivia(content, 0, content.len());

    let body_open = if active_token_at(content, code, first_token, "let") {
        returned_attrset_after_let(content, code, first_token)?
    } else {
        let first_open = (0..bytes.len())
            .find(|&i| bytes[i] == b'{' && code[i])
            .ok_or("Nix module attribute set not found")?;
        let first_close = find_matching_delimiter(content, first_open, '{', '}')
            .ok_or("Nix module attribute set is not balanced")?;
        let after_first = skip_nix_trivia(content, first_close + 1, content.len());

        let colon = if bytes.get(after_first) == Some(&b':') && code[after_first] {
            Some(after_first)
        } else if bytes.get(after_first) == Some(&b'@') && code[after_first] {
            let argument_start = skip_nix_trivia(content, after_first + 1, content.len());
            let argument_end = read_nix_identifier_end(content, argument_start, content.len());
            let colon = skip_nix_trivia(content, argument_end, content.len());
            (argument_end > argument_start && bytes.get(colon) == Some(&b':') && code[colon])
                .then_some(colon)
        } else {
            None
        };

        if let Some(colon) = colon {
            let expression_start = skip_nix_trivia(content, colon + 1, content.len());
            if active_token_at(content, code, expression_start, "let") {
                returned_attrset_after_let(content, code, expression_start)?
            } else {
                (expression_start..bytes.len())
                    .find(|&i| bytes[i] == b'{' && code[i])
                    .ok_or("returned Nix module attribute set not found")?
            }
        } else {
            first_open
        }
    };

    let body_close = find_matching_delimiter(content, body_open, '{', '}')
        .ok_or("returned Nix module attribute set is not balanced")?;
    Ok((body_open, body_close))
}

/// Locate the attribute set returned by a top-level `let ... in { ... }` expression.
fn returned_attrset_after_let(
    content: &str,
    code: &[bool],
    let_start: usize,
) -> Result<usize, String> {
    let in_pos = find_top_level_keyword(content, code, let_start + "let".len(), "in")
        .ok_or("Nix let expression has no top-level 'in'")?;
    let body = skip_nix_trivia(content, in_pos + "in".len(), content.len());
    if content.as_bytes().get(body) != Some(&b'{') || !code[body] {
        return Err("Nix let expression does not return an attribute set".to_string());
    }
    Ok(body)
}

/// Find a Nix keyword outside nested delimiters, comments, and strings.
fn find_top_level_keyword(
    content: &str,
    code: &[bool],
    start: usize,
    keyword: &str,
) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut braces = 0usize;
    let mut brackets = 0usize;
    let mut parentheses = 0usize;
    let mut cursor = start;

    while cursor < bytes.len() {
        if braces == 0
            && brackets == 0
            && parentheses == 0
            && active_token_at(content, code, cursor, keyword)
        {
            return Some(cursor);
        }

        if code[cursor] {
            match bytes[cursor] {
                b'{' => braces += 1,
                b'}' => braces = braces.saturating_sub(1),
                b'[' => brackets += 1,
                b']' => brackets = brackets.saturating_sub(1),
                b'(' => parentheses += 1,
                b')' => parentheses = parentheses.saturating_sub(1),
                _ => {}
            }
        }
        cursor += content[cursor..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(1);
    }

    None
}

/// Return whether a token occurs at the direct attribute-set level of the module body.
fn is_direct_module_attribute(
    content: &str,
    code: &[bool],
    body_open: usize,
    token_start: usize,
) -> bool {
    let mut depth = 0usize;
    for index in body_open..token_start {
        if !code[index] {
            continue;
        }
        match content.as_bytes()[index] {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth == 1
}

/// Find a direct assignment to a parent attribute path of a service option.
fn find_direct_prefix_assignment(
    content: &str,
    code: &[bool],
    body_start: usize,
    body_end: usize,
    attribute: &str,
) -> Option<String> {
    let option_path = attribute.strip_suffix(".enable")?;
    let prefixes = if let Some(service) = option_path.strip_prefix("services.") {
        vec!["services".to_string(), format!("services.{service}")]
    } else if let Some(service) = option_path.strip_prefix("systemd.services.") {
        vec![
            "systemd".to_string(),
            "systemd.services".to_string(),
            format!("systemd.services.{service}"),
        ]
    } else {
        return None;
    };

    for prefix in prefixes {
        let mut search_from = body_start;
        while search_from < body_end {
            let Some(relative) = content[search_from..body_end].find(&prefix) else {
                break;
            };
            let start = search_from + relative;
            let end = start + prefix.len();
            search_from = end;

            if !active_token_at(content, code, start, &prefix)
                || !is_direct_module_attribute(content, code, body_start, start)
            {
                continue;
            }

            let equals = skip_nix_trivia(content, end, body_end);
            let next = content.as_bytes().get(equals + 1).copied();
            if equals < body_end
                && content.as_bytes()[equals] == b'='
                && code[equals]
                && !matches!(next, Some(b'=' | b'>'))
            {
                return Some(prefix);
            }
        }
    }

    None
}

/// Find a unique, directly declared standalone boolean assignment.
fn find_boolean_assignment(
    content: &str,
    attribute: &str,
) -> Result<Option<(usize, usize)>, String> {
    let code = nix_code_mask(content);
    let (body_start, body_end) = returned_module_attrset(content, &code)?;
    let mut assignments = Vec::new();
    let mut nested_assignment_found = false;
    let mut search_from = body_start;

    while search_from < body_end {
        let Some(relative) = content[search_from..body_end].find(attribute) else {
            break;
        };
        let start = search_from + relative;
        let end = start + attribute.len();
        search_from = end;

        // Match the complete attribute path, not a prefix such as
        // services.openssh.enableExtra, and ignore comments/strings.
        if !active_token_at(content, &code, start, attribute) {
            continue;
        }

        let equals = skip_nix_trivia(content, end, body_end);
        if !is_direct_module_attribute(content, &code, body_start, start) {
            if equals < body_end
                && content.as_bytes()[equals] == b'='
                && code[equals]
                && content.as_bytes().get(equals + 1) != Some(&b'=')
                && content.as_bytes().get(equals + 1) != Some(&b'>')
            {
                nested_assignment_found = true;
            }
            continue;
        }
        if equals >= content.len() || content.as_bytes()[equals] != b'=' || !code[equals] {
            continue;
        }

        let next = content.as_bytes().get(equals + 1).copied();
        if matches!(next, Some(b'=' | b'>')) {
            continue;
        }

        assignments.push(equals);
    }

    if let Some(prefix) =
        find_direct_prefix_assignment(content, &code, body_start, body_end, attribute)
    {
        return Err(format!(
            "found direct assignment to '{}' before '{}'; refusing to add a conflicting declaration",
            prefix, attribute
        ));
    }

    match assignments.as_slice() {
        [] if nested_assignment_found => Err(format!(
            "found '{}' only in a nested attribute set; refusing to add a conflicting top-level declaration",
            attribute
        )),
        [] => Ok(None),
        [equals] => {
            let value_start = skip_nix_trivia(content, equals + 1, body_end);

            let value_end = if active_token_at(content, &code, value_start, "true") {
                value_start + "true".len()
            } else if active_token_at(content, &code, value_start, "false") {
                value_start + "false".len()
            } else {
                return Err(format!(
                    "existing assignment to '{}' uses an unsupported expression; refusing to edit it or append a conflicting declaration",
                    attribute
                ));
            };

            let terminator = skip_nix_trivia(content, value_end, content.len());
            if terminator >= content.len()
                || content.as_bytes()[terminator] != b';'
                || !code[terminator]
            {
                return Err(format!(
                    "existing assignment to '{}' is not a standalone true/false literal; refusing to edit it or append a conflicting declaration",
                    attribute
                ));
            }

            Ok(Some((value_start, value_end)))
        }
        _ => Err(format!(
            "found multiple assignments to '{}'; refusing to guess which definition to edit",
            attribute
        )),
    }
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
    fn supports_nested_package_attribute_paths() {
        let content = "{ environment.systemPackages = with pkgs; [ firefox ]; }";
        let result = add_package_to_content(content, "kdePackages.okular").unwrap();

        assert!(result.contains("[ firefox kdePackages.okular ]"));
        let removed = remove_package_from_content(&result, "kdePackages.okular").unwrap();
        assert!(removed.contains("[ firefox  ]"));
        assert!(!removed.contains("kdePackages.okular"));
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
    #[test]
    fn rejects_service_names_that_are_not_simple_nix_identifiers() {
        for service in ["foo.bar", "foo; services.nginx.enable = true", "if", ""] {
            assert!(add_service_to_content("{ }", service, true).is_err());
        }
    }

    #[test]
    fn refuses_to_add_a_top_level_option_when_only_nested_option_exists() {
        let content =
            "{\n  home-manager.users.bob = {\n    services.foo.enable = false;\n  };\n}\n";
        let error = add_service_to_content(content, "foo", true).unwrap_err();

        assert!(error.contains("only in a nested attribute set"));
        assert!(content.contains("services.foo.enable = false;"));
        assert!(!content.contains("services.foo.enable = true;"));
    }

    #[test]
    fn finds_the_returned_attribute_set_after_a_let_expression() {
        let content = "let helper = 1; in {\n  services.foo.enable = false;\n}\n";
        let result = add_service_to_content(content, "foo", true).unwrap();

        assert!(result.contains("services.foo.enable = true;"));
        assert!(result.contains("let helper = 1;"));
    }

    #[test]
    fn skips_attribute_sets_inside_let_bindings() {
        let content = "let helper = { nested = { services.foo.enable = false; }; }; in {\n  services.foo.enable = false;\n}\n";
        let result = add_service_to_content(content, "foo", true).unwrap();

        assert!(result.contains("nested = { services.foo.enable = false; }"));
        assert!(result.contains("services.foo.enable = true;"));
    }

    #[test]
    fn edits_direct_option_when_nested_option_appears_first() {
        let content = "{\n  home-manager.users.bob = {\n    services.foo.enable = false;\n  };\n  services.foo.enable = false;\n}\n";
        let result = add_service_to_content(content, "foo", true).unwrap();

        assert!(result.contains("home-manager.users.bob = {\n    services.foo.enable = false;"));
        assert!(result.contains("services.foo.enable = true;"));
    }

    #[test]
    fn refuses_to_append_over_a_mkforce_service_definition() {
        let content = "{\n  services.openssh.enable = lib.mkForce true;\n}\n";
        let result = add_service_to_content(content, "openssh", false);

        let error = result.unwrap_err();
        assert!(error.contains("unsupported expression"));
        assert!(error.contains("services.openssh.enable"));
        assert!(content.contains("lib.mkForce true"));
        assert!(!content.contains("services.openssh.enable = false;"));
    }

    #[test]
    fn refuses_to_edit_a_conditional_service_definition() {
        let content = "{\n  services.openssh.enable = if condition then true else false;\n}\n";
        let result = add_service_to_content(content, "openssh", false);

        assert!(result.unwrap_err().contains("unsupported expression"));
        assert!(content.contains("if condition then true else false"));
    }

    #[test]
    fn service_assignment_match_requires_exact_attribute_boundary() {
        let content = "{\n  services.openssh.enableExtra = true;\n}\n";
        let result = add_service_to_content(content, "openssh", false).unwrap();

        assert!(result.contains("services.openssh.enableExtra = true;"));
        assert!(result.contains("services.openssh.enable = false;"));
    }

    #[test]
    fn ignores_service_option_mentions_in_comments_and_strings() {
        let content = r#"{
  # services.openssh.enable = true;
  description = "services.openssh.enable = true;";
}
"#;
        let result = add_service_to_content(content, "openssh", false).unwrap();

        assert!(result.contains("# services.openssh.enable = true;"));
        assert!(result.contains("\"services.openssh.enable = true;\""));
        assert!(result.contains("services.openssh.enable = false;"));
    }

    #[test]
    fn refuses_to_edit_duplicate_service_assignments() {
        let content =
            "{\n  services.openssh.enable = true;\n  services.openssh.enable = false;\n}\n";
        let result = add_service_to_content(content, "openssh", true);

        assert!(result.unwrap_err().contains("multiple assignments"));
    }

    #[test]
    fn refuses_to_edit_a_boolean_followed_by_an_expression() {
        let content = "{\n  services.openssh.enable = true && condition;\n}\n";
        let result = add_service_to_content(content, "openssh", false);

        assert!(
            result
                .unwrap_err()
                .contains("standalone true/false literal")
        );
    }

    #[test]
    fn rejects_parent_attribute_assignments_that_would_conflict_with_service_options() {
        let cases = [
            "{ services.openssh = { enable = true; }; }",
            "{ services = { openssh.enable = true; }; }",
            "{ systemd.services.openssh = { enable = true; }; }",
            "{ systemd.services = { openssh.enable = true; }; }",
            "{ systemd = { services.openssh.enable = true; }; }",
        ];

        for content in cases {
            let error = add_service_to_content(content, "openssh", false).unwrap_err();
            assert!(
                error.contains("refusing"),
                "unsafe parent declaration was not rejected for {content:?}: {error}"
            );
        }
    }

    #[test]
    fn supports_module_function_attribute_set_aliased_after_closing_brace() {
        let content = "{ config, pkgs, ... }@args: {\\n  services.openssh.enable = true;\\n}\\n";
        let result = add_service_to_content(content, "openssh", false).unwrap();

        assert!(result.contains("services.openssh.enable = false;"));
        assert!(!result.contains("services.openssh.enable = true;"));
    }

    #[test]
    fn selects_the_existing_service_option_when_both_option_paths_exist() {
        let content =
            "{ services.openssh.enable = false; systemd.services.openssh.enable = true; }";
        assert_eq!(
            service_option_for_content(content, "openssh").unwrap(),
            "services.openssh.enable"
        );

        let raw_systemd = "{ systemd.services.custom.enable = false; }";
        assert_eq!(
            service_option_for_content(raw_systemd, "custom").unwrap(),
            "systemd.services.custom.enable"
        );
    }

    #[test]
    fn adds_and_removes_dotted_package_attribute_paths() {
        let original = "environment.systemPackages = with pkgs; [\n  firefox\n];\n";
        let updated = add_package_to_content(original, "python3Packages.requests").unwrap();
        assert!(updated.contains("python3Packages.requests"));

        let removed = remove_package_from_content(&updated, "python3Packages.requests").unwrap();
        assert!(!removed.contains("python3Packages.requests"));
        assert!(removed.contains("firefox"));
    }
}
