use sample_discord_bot::usecase::dto::DiscordExecStep;
use sample_discord_bot::usecase::slash_commands::help as help_usecase;

#[test]
fn test_help_command_plan() {
    let plan = help_usecase::execute().expect("help usecase should return plan");
    let steps = plan.into_steps();

    assert_eq!(steps.len(), 1, "help should return single step");

    match &steps[0] {
        DiscordExecStep::Response(payload) => {
            assert_eq!(
                payload.content.as_deref(),
                Some("利用可能なコマンド: /help"),
                "help response content should match"
            );
            assert!(
                payload.ephemeral.is_none(),
                "help should not force ephemeral"
            );
        }
        other => panic!("Expected Response step, got {:?}", other),
    }
}
