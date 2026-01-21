use crate::domain::policy::greeting::is_greeting;
use crate::usecase::dto::output::discord_exec::validate_plan;
use crate::usecase::dto::{
    DiscordExecPlan, DiscordExecStep, MessageInput, MessagePayload, PlanValidationError,
};
use validate_macro::sync_validate_return;

#[sync_validate_return(validate_plan)]
pub fn execute(input: MessageInput) -> Result<DiscordExecPlan, PlanValidationError> {
    if !is_greeting(&input.content) {
        return Ok(DiscordExecPlan::new(vec![]));
    }
    // 返信メッセージのペイロードを作成
    let message_payload = MessagePayload {
        content: "おはようございます！".to_string().into(),
        ..Default::default()
    };
    // 実行ステップを作成
    let exec_step = DiscordExecStep::Send {
        channel_id: input.channel_id,
        payload: message_payload,
    };
    // 実行ステップでプランを作成して返す
    Ok(DiscordExecPlan::new(vec![exec_step]))
}

#[cfg(test)]
mod tests {
    use super::execute;
    use crate::domain::policy::greeting::GREETING_TEXT;
    use crate::usecase::dto::{DiscordExecStep, MessageInput};

    #[test]
    fn returns_output_for_greeting() {
        let input = MessageInput::new(GREETING_TEXT, 1);
        let output = execute(input).expect("expected plan");
        let step = output
            .into_steps()
            .into_iter()
            .next()
            .expect("expected exec step");
        match step {
            DiscordExecStep::Send {
                channel_id,
                payload,
            } => {
                assert_eq!(channel_id, 1);
                assert_eq!(payload.content.as_deref(), Some("おはようございます！"));
            }
            other => panic!("unexpected exec step: {:?}", other),
        }
    }

    #[test]
    fn returns_empty_plan_for_other_text() {
        let input = MessageInput::new("こんばんは", 1);
        let output = execute(input).expect("expected plan");
        assert!(output.steps().is_empty());
    }
}
