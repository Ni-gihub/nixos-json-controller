use super::{
    parser,
    Action,
};



#[test]
fn parse_install_package() {

  let json = r#"
        {
            "action":"install_package",
            "target":"firefox"
        }
        "#;


        let command = parser::parse(json)
            .unwrap();


        assert!(
            matches!(
                command.action,
                Action::InstallPackage
            )
        );


        assert_eq!(
            command.target.raw,
            "firefox"
        );
    }

#[test]
fn parse_enable_service(){

    let json = r#"
    {
        "action":"enable_service",
        "target":"openssh"
    }
    "#;


    let command =
        parser::parse(json)
        .unwrap();


    assert!(
        matches!(
            command.action,
            Action::EnableService
        )
    );


    assert_eq!(
        command.target.raw,
        "openssh"
    );
}

#[test]
fn reject_invalid_action(){

    let json = r#"
    {
        "action":"install",
        "target":"firefox"
    }
    "#;


    let result =
        parser::parse(json);


    assert!(
        result.is_err()
    );

}

#[test]
fn reject_missing_target(){

    let json = r#"
    {
        "action":"install_package"
    }
    "#;


    let result =
        parser::parse(json);


    assert!(
        result.is_err()
    );

}

#[test]
fn parse_old_style_action_name() {

    let json = r#"
    {
        "action":"installpackage",
        "target":"firefox"
    }
    "#;


    let command = parser::parse(json)
        .unwrap();


    assert_eq!(
        command.action,
        Action::InstallPackage
    );

}