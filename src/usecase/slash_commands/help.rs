use crate::usecase::dto::output::discord_exec::validate_plan;
use crate::usecase::dto::{DiscordExecPlan, DiscordExecStep, MessagePayload, PlanValidationError};
use validate_macro::sync_validate_return;

#[sync_validate_return(validate_plan)]
pub fn execute() -> Result<DiscordExecPlan, PlanValidationError> {
    let message_payload = MessagePayload {
        content: Some("利用可能なコマンド: /help".into()),
        ..Default::default()
    };
    let exec_step = DiscordExecStep::Response(message_payload);

    Ok(DiscordExecPlan::new(vec![exec_step]))
}
