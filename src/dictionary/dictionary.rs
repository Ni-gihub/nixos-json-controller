use std::collections::HashMap;

use super::{
    package::PackageDictionary,
    service::ServiceDictionary,
};

const PACKAGES_JSON: &str =
    include_str!("packages.json");

const SERVICES_JSON: &str =
    include_str!("services.json");

pub struct Dictionary {
    packages: PackageDictionary,
    services: ServiceDictionary,
}

impl Dictionary {
    pub fn load() -> Result<Self, String> {
        let packages =
            serde_json::from_str(
                PACKAGES_JSON
            )
            .map_err(|e| e.to_string())?;

        let services =
            serde_json::from_str(
                SERVICES_JSON
            )
            .map_err(|e| e.to_string())?;

        Ok(Self {
            packages,
            services,
        })
    }

    pub fn resolve_package(
        &self,
        input: &str,
    ) -> Option<&str> {
        resolve(
            &self.packages,
            input,
        )
    }

    pub fn resolve_service(
        &self,
        input: &str,
    ) -> Option<&str> {
        resolve(
            &self.services,
            input,
        )
    }
}

fn resolve<'a>(
    dictionary: &'a HashMap<String, Vec<String>>,
    input: &str,
) -> Option<&'a str> {
    dictionary
        .iter()
        .find_map(|(canonical, aliases)| {
            aliases
                .iter()
                .any(|alias| alias == input)
                .then_some(canonical.as_str())
        })
}




#[cfg(test)]
mod tests {
    use super::Dictionary;

    #[test]
    fn resolve_package_alias() {
        let dictionary =
            Dictionary::load()
                .unwrap();

        assert_eq!(
            dictionary
                .resolve_package("firefox"),
            Some("firefox")
        );

        assert_eq!(
            dictionary
                .resolve_package("ファイアフォックス"),
            Some("firefox")
        );

        assert_eq!(
            dictionary
                .resolve_package("火狐"),
            Some("firefox")
        );
    }

    #[test]
    fn resolve_service_alias() {
        let dictionary =
            Dictionary::load()
                .unwrap();

        assert_eq!(
            dictionary
                .resolve_service("ssh"),
            Some("openssh")
        );

        assert_eq!(
            dictionary
                .resolve_service("sshd"),
            Some("openssh")
        );
    }

    #[test]
    fn unknown_package_returns_none() {
        let dictionary =
            Dictionary::load()
                .unwrap();

        assert_eq!(
            dictionary
                .resolve_package("unknown-package"),
            None
        );
    }

    #[test]
    fn resolve_more_package_aliases() {
        let dictionary =
            Dictionary::load()
                .unwrap();

        assert_eq!(
            dictionary.resolve_package("chrome"),
            Some("google-chrome")
        );

        assert_eq!(
            dictionary.resolve_package("vs code"),
            Some("vscode")
        );

        assert_eq!(
            dictionary.resolve_package("python"),
            Some("python3")
        );

        assert_eq!(
            dictionary.resolve_package("git"),
            Some("git")
        );
    }
}