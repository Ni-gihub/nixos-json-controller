use crate::{
    command::{
        Action,
        Target,
    },
    dictionary::Dictionary,
};

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub name: String,
}

pub struct Resolver;

impl Resolver {
    pub fn resolve(
        action: Action,
        target: Target,
    ) -> Result<ResolvedTarget, String> {
        let dictionary =
            Dictionary::load()?;

        let name =
            match action {
                Action::InstallPackage
                | Action::RemovePackage => {
                    dictionary
                        .resolve_package(
                            &target.raw
                        )
                }

                Action::EnableService
                | Action::DisableService => {
                    dictionary
                        .resolve_service(
                            &target.raw
                        )
                }
            }
            .ok_or_else(|| {
                format!(
                    "unknown target: {}",
                    target.raw
                )
            })?;

        Ok(
            ResolvedTarget {
                name: name.to_string(),
            }
        )
    }
}