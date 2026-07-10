use serde::Deserialize;


#[derive(Debug, Deserialize, PartialEq)]
pub enum Action {


    #[serde(alias = "install_package")]
    #[serde(rename = "installpackage")]
    InstallPackage,


    #[serde(alias = "remove_package")]
    #[serde(rename = "removepackage")]
    RemovePackage,


    #[serde(alias = "enable_service")]
    #[serde(rename = "enableservice")]
    EnableService,


    #[serde(alias = "disable_service")]
    #[serde(rename = "disableservice")]
    DisableService,

}