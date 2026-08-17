use super::flake;


// ============================================================
// Package
// ============================================================

pub fn add_package(
    package: &str,
) -> Result<String, String> {

    let content =
        flake::read_pkgs()?;

    add_package_to_content(
        &content,
        package,
    )
}


pub fn remove_package(
    package: &str,
) -> Result<String, String> {

    let content =
        flake::read_pkgs()?;

    remove_package_from_content(
        &content,
        package,
    )
}


// ============================================================
// Service
// ============================================================

pub fn enable_service(
    service: &str,
) -> Result<String, String> {

    let content =
        flake::read_core()?;

    add_service_to_content(
        &content,
        service,
        true,
    )
}


pub fn disable_service(
    service: &str,
) -> Result<String, String> {

    let content =
        flake::read_core()?;

    add_service_to_content(
        &content,
        service,
        false,
    )
}


// ============================================================
// Package content manipulation
// ============================================================

pub fn add_package_to_content(
    content: &str,
    package: &str,
) -> Result<String, String> {

    let marker =
        "environment.systemPackages = with pkgs; [";

    let position =
        content
            .find(marker)
            .ok_or(
                "environment.systemPackages section not found"
            )?;

    let insert_position =
        position + marker.len();

    let mut result =
        content.to_string();

    result.insert_str(
        insert_position,
        &format!(
            "\n  {}",
            package
        )
    );

    Ok(result)
}


pub fn remove_package_from_content(
    content: &str,
    package: &str,
) -> Result<String, String> {

    let mut result =
        String::new();

    let mut found =
        false;

    for current_line in content.lines() {

        if current_line.trim() == package {

            found = true;

            continue;
        }

        result.push_str(
            current_line
        );

        result.push('\n');
    }

    if !found {

        return Err(
            format!(
                "package '{}' not found",
                package
            )
        );
    }

    if content.ends_with('\n') {

        Ok(result)

    } else {

        Ok(
            result
                .trim_end_matches('\n')
                .to_string()
        )
    }
}


// ============================================================
// Service content manipulation
// ============================================================

pub fn add_service_to_content(
    content: &str,
    service: &str,
    enabled: bool,
) -> Result<String, String> {

    let setting =
        format!(
            "systemd.services.{}.enable = {};",
            service,
            enabled
        );

    let prefix =
        format!(
            "systemd.services.{}.enable = ",
            service
        );

    let mut result =
        String::new();

    let mut found =
        false;

    for line in content.lines() {

        if line
            .trim_start()
            .starts_with(&prefix)
        {

            result.push_str(
                &format!(
                    "  {}\n",
                    setting
                )
            );

            found = true;

        } else {

            result.push_str(
                line
            );

            result.push('\n');
        }
    }

    if found {

        return Ok(result);
    }

    let closing_brace =
        content
            .rfind('}')
            .ok_or(
                "Nix module closing brace not found"
            )?;

    let mut result =
        content.to_string();

    result.insert_str(
        closing_brace,
        &format!(
            "  {}\n",
            setting
        )
    );

    Ok(result)
}