#![allow(dead_code)]

use super::error::validation::PlanValidationError;

#[derive(Debug, Clone)]
pub struct DiscordExecPlan {
    steps: Vec<DiscordExecStep>,
}

impl DiscordExecPlan {
    pub fn new(steps: Vec<DiscordExecStep>) -> Self {
        Self { steps }
    }

    pub fn steps(&self) -> &[DiscordExecStep] {
        &self.steps
    }

    pub fn into_steps(self) -> Vec<DiscordExecStep> {
        self.steps
    }
}

#[derive(Debug, Clone)]
pub enum DiscordExecStep {
    Send {
        channel_id: u64,
        payload: MessagePayload,
    },
    Defer(DeferPayload),
    Response(MessagePayload),
    EditOriginal(MessagePayload),
    FollowUp(MessagePayload),
    OpenModal(ModalPayload),
}

pub fn validate_plan(plan: &DiscordExecPlan) -> Result<(), PlanValidationError> {
    validate_steps(&plan.steps)
}

fn validate_steps(steps: &[DiscordExecStep]) -> Result<(), PlanValidationError> {
    if steps.is_empty() {
        return Ok(());
    }

    let has_send = steps
        .iter()
        .any(|step| matches!(step, DiscordExecStep::Send { .. }));
    let has_interaction = steps
        .iter()
        .any(|step| !matches!(step, DiscordExecStep::Send { .. }));

    if has_send && has_interaction {
        return Err(PlanValidationError::MixedSendAndInteraction);
    }

    if has_send {
        return Ok(());
    }

    let first = &steps[0];
    if !matches!(
        first,
        DiscordExecStep::Defer(_) | DiscordExecStep::Response(_) | DiscordExecStep::OpenModal(_)
    ) {
        return Err(PlanValidationError::InvalidFirstStep);
    }

    if steps
        .iter()
        .skip(1)
        .any(|step| matches!(step, DiscordExecStep::Response(_)))
    {
        return Err(PlanValidationError::ResponseNotFirst);
    }

    if steps
        .iter()
        .skip(1)
        .any(|step| matches!(step, DiscordExecStep::Defer(_)))
    {
        return Err(PlanValidationError::DeferNotFirst);
    }

    let has_defer = steps
        .iter()
        .any(|step| matches!(step, DiscordExecStep::Defer(_)));
    let has_response = steps
        .iter()
        .any(|step| matches!(step, DiscordExecStep::Response(_)));
    if has_defer && has_response {
        return Err(PlanValidationError::DeferAndResponse);
    }

    if steps.len() > 1
        && steps
            .iter()
            .any(|step| matches!(step, DiscordExecStep::OpenModal(_)))
    {
        return Err(PlanValidationError::OpenModalNotExclusive);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::error::validation::PlanValidationError;
    use super::{validate_plan, DiscordExecPlan, DiscordExecStep, MessagePayload};

    #[test]
    fn allows_empty_plan() {
        let plan = DiscordExecPlan::new(vec![]);
        assert!(validate_plan(&plan).is_ok());
    }

    #[test]
    fn rejects_mixed_send_and_interaction() {
        let steps = vec![
            DiscordExecStep::Send {
                channel_id: 1,
                payload: MessagePayload::default(),
            },
            DiscordExecStep::Response(MessagePayload::default()),
        ];
        let err =
            validate_plan(&DiscordExecPlan::new(steps)).expect_err("expected validation error");
        assert_eq!(err, PlanValidationError::MixedSendAndInteraction);
    }

    #[test]
    fn rejects_response_not_first() {
        let steps = vec![
            DiscordExecStep::Defer(super::DeferPayload::public()),
            DiscordExecStep::Response(MessagePayload::default()),
        ];
        let err =
            validate_plan(&DiscordExecPlan::new(steps)).expect_err("expected validation error");
        assert_eq!(err, PlanValidationError::ResponseNotFirst);
    }

    #[test]
    fn rejects_open_modal_with_other_steps() {
        let steps = vec![
            DiscordExecStep::OpenModal(super::ModalPayload::new("id", "title", vec![])),
            DiscordExecStep::FollowUp(MessagePayload::default()),
        ];
        let err =
            validate_plan(&DiscordExecPlan::new(steps)).expect_err("expected validation error");
        assert_eq!(err, PlanValidationError::OpenModalNotExclusive);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeferPayload {
    pub ephemeral: bool,
}

impl DeferPayload {
    pub fn public() -> Self {
        Self { ephemeral: false }
    }

    pub fn ephemeral() -> Self {
        Self { ephemeral: true }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MessagePayload {
    pub content: Option<String>,
    pub embeds: Option<Vec<EmbedPayload>>,
    pub components: Option<Vec<ActionRowPayload>>,
    pub ephemeral: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct EmbedPayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<EmbedFieldPayload>,
}

impl EmbedPayload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn field(mut self, field: EmbedFieldPayload) -> Self {
        self.fields.push(field);
        self
    }
}

#[derive(Debug, Clone)]
pub struct EmbedFieldPayload {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

impl EmbedFieldPayload {
    pub fn new(name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            inline,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionRowPayload {
    Buttons(Vec<ButtonPayload>),
    SelectMenu(SelectMenuPayload),
    InputText(TextInputPayload),
}

#[derive(Debug, Clone)]
pub enum ButtonStylePayload {
    Primary,
    Secondary,
    Success,
    Danger,
    Link,
}

#[derive(Debug, Clone)]
pub struct ButtonPayload {
    pub style: ButtonStylePayload,
    pub label: Option<String>,
    pub custom_id: Option<String>,
    pub url: Option<String>,
    pub disabled: bool,
}

impl ButtonPayload {
    pub fn new(custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            style: ButtonStylePayload::Primary,
            label: Some(label.into()),
            custom_id: Some(custom_id.into()),
            url: None,
            disabled: false,
        }
    }

    pub fn link(url: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            style: ButtonStylePayload::Link,
            label: Some(label.into()),
            custom_id: None,
            url: Some(url.into()),
            disabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectMenuPayload {
    pub custom_id: String,
    pub options: Vec<SelectOptionPayload>,
    pub placeholder: Option<String>,
    pub min_values: Option<u8>,
    pub max_values: Option<u8>,
    pub disabled: bool,
}

impl SelectMenuPayload {
    pub fn new(custom_id: impl Into<String>, options: Vec<SelectOptionPayload>) -> Self {
        Self {
            custom_id: custom_id.into(),
            options,
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectOptionPayload {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub default: bool,
}

impl SelectOptionPayload {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            default: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModalPayload {
    pub custom_id: String,
    pub title: String,
    pub inputs: Vec<TextInputPayload>,
}

impl ModalPayload {
    pub fn new(
        custom_id: impl Into<String>,
        title: impl Into<String>,
        inputs: Vec<TextInputPayload>,
    ) -> Self {
        Self {
            custom_id: custom_id.into(),
            title: title.into(),
            inputs,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextInputStylePayload {
    Short,
    Paragraph,
}

#[derive(Debug, Clone)]
pub struct TextInputPayload {
    pub custom_id: String,
    pub label: String,
    pub style: TextInputStylePayload,
    pub placeholder: Option<String>,
    pub min_length: Option<u16>,
    pub max_length: Option<u16>,
    pub required: bool,
    pub value: Option<String>,
}

impl TextInputPayload {
    pub fn new(
        custom_id: impl Into<String>,
        label: impl Into<String>,
        style: TextInputStylePayload,
    ) -> Self {
        Self {
            custom_id: custom_id.into(),
            label: label.into(),
            style,
            placeholder: None,
            min_length: None,
            max_length: None,
            required: true,
            value: None,
        }
    }
}
