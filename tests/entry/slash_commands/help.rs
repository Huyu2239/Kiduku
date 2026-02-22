use kiduku::usecase::slash_commands::help as help_usecase;

#[test]
fn test_help_command_output() {
    let help = help_usecase::execute().expect("help usecase should return output");

    assert_eq!(help.title, "Kiduku ヘルプ");
    assert!(
        help.description.contains("✅"),
        "help description should explain the reaction"
    );

    let command_names = help
        .commands
        .iter()
        .map(|command| command.name.as_str())
        .collect::<Vec<_>>();

    assert!(command_names.contains(&"/help"));
    assert!(command_names.contains(&"/check-reads"));
}
