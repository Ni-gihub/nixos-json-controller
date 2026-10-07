use crate::{
    command::{Action, Target},
    dictionary::Dictionary,
};

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub name: String,
}

pub struct Resolver;

impl Resolver {
    pub fn resolve(action: Action, target: Target) -> Result<ResolvedTarget, String> {
        let dictionary = Dictionary::load()?;

        let name = match action {
            Action::InstallPackage | Action::RemovePackage => dictionary
                .resolve_package(&target.raw)
                .or_else(|| is_safe_package_reference(&target.raw).then_some(target.raw.as_str())),

            Action::EnableService | Action::DisableService => {
                dictionary.resolve_service(&target.raw)
            }
        }
        .ok_or_else(|| format!("unknown target: {}", target.raw))?;

        Ok(ResolvedTarget {
            name: name.to_string(),
        })
    }
}


fn is_safe_package_reference(value: &str) -> bool {
    !value.is_empty()
        && value
            .split('.')
            .all(|segment| is_safe_package_segment(segment))
}

fn is_safe_package_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '_' | '-' | '\'')
        })
        && !matches!(
            segment,
            "assert" | "else" | "if" | "in" | "inherit" | "let" | "or" | "rec" | "then" | "with"
        )
}

#[cfg(test)]
mod package_reference_tests {
    use super::is_safe_package_reference;

    #[test]
    fn accepts_nixpkgs_attribute_paths() {
        assert!(is_safe_package_reference("firefox"));
        assert!(is_safe_package_reference("python3Packages.requests"));
    }

    #[test]
    fn rejects_nix_expressions() {
        assert!(!is_safe_package_reference("firefox; builtins.abort"));
        assert!(!is_safe_package_reference("firefox $(touch /tmp/pwned)"));
    }
}
