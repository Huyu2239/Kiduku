use anyhow::{anyhow, bail, Context as _};
use poise::serenity_prelude as serenity;

use crate::presentation::{Context, Error};
use crate::usecase::dto::{
    ActionRowPayload, ButtonPayload, ButtonStylePayload, DeferPayload, DiscordExecPlan,
    DiscordExecStep, EmbedPayload, MessagePayload, ModalPayload, SelectMenuPayload,
    SelectOptionPayload, TextInputPayload, TextInputStylePayload,
};

pub async fn execute(ctx: &serenity::Context, plan: DiscordExecPlan) -> Result<(), Error> {
    for step in plan.into_steps() {
        match step {
            DiscordExecStep::Send {
                channel_id,
                payload,
            } => {
                let message = build_message_payload(payload)?;
                serenity::ChannelId::new(channel_id)
                    .send_message(&ctx.http, message)
                    .await
                    .context("failed to send message")?;
            }
            _ => {
                bail!("unsupported step");
            }
        }
    }
    Ok(())
}

pub async fn execute_from_interaction(
    ctx: Context<'_>,
    plan: DiscordExecPlan,
) -> Result<(), Error> {
    for step in plan.into_steps() {
        match step {
            DiscordExecStep::Defer(spec) => execute_defer(&ctx, spec).await?,
            DiscordExecStep::Response(spec) => execute_response(&ctx, spec).await?,
            DiscordExecStep::EditOriginal(spec) => execute_edit_original(&ctx, spec).await?,
            DiscordExecStep::FollowUp(spec) => execute_follow_up(&ctx, spec).await?,
            DiscordExecStep::OpenModal(spec) => execute_open_modal(&ctx, spec).await?,
            _ => {
                bail!("unsupported step");
            }
        }
    }
    Ok(())
}

async fn execute_defer(ctx: &Context<'_>, spec: DeferPayload) -> Result<(), Error> {
    if spec.ephemeral {
        ctx.defer_ephemeral().await?;
    } else {
        ctx.defer().await?;
    }
    Ok(())
}

async fn execute_response(ctx: &Context<'_>, spec: MessagePayload) -> Result<(), Error> {
    let builder = build_interaction_payload(spec)?;
    ctx.send(builder).await?;
    Ok(())
}

async fn execute_edit_original(ctx: &Context<'_>, spec: MessagePayload) -> Result<(), Error> {
    if spec.ephemeral == Some(true) {
        bail!("ephemeral is not supported for edit_original");
    }
    let builder = build_interaction_payload(spec)?;
    match ctx {
        Context::Application(app_ctx) => {
            let edit =
                builder.to_slash_initial_response_edit(serenity::EditInteractionResponse::new());
            app_ctx
                .interaction
                .edit_response(app_ctx, edit)
                .await
                .context("failed to edit original interaction response")?;
            Ok(())
        }
        Context::Prefix(_) => bail!("edit_original is only supported for application commands"),
    }
}

async fn execute_follow_up(ctx: &Context<'_>, spec: MessagePayload) -> Result<(), Error> {
    let builder = build_interaction_payload(spec)?;
    match ctx {
        Context::Application(app_ctx) => {
            let followup = builder
                .to_slash_followup_response(serenity::CreateInteractionResponseFollowup::new());
            app_ctx
                .interaction
                .create_followup(app_ctx, followup)
                .await
                .context("failed to create followup response")?;
            Ok(())
        }
        Context::Prefix(_) => {
            ctx.send(builder).await?;
            Ok(())
        }
    }
}

async fn execute_open_modal(ctx: &Context<'_>, spec: ModalPayload) -> Result<(), Error> {
    let modal = build_modal(spec)?;
    match ctx {
        Context::Application(app_ctx) => {
            let response = serenity::CreateInteractionResponse::Modal(modal);
            app_ctx
                .interaction
                .create_response(app_ctx, response)
                .await
                .context("failed to open modal")?;
            Ok(())
        }
        Context::Prefix(_) => bail!("open_modal is only supported for application commands"),
    }
}

fn build_interaction_payload(message_payload: MessagePayload) -> Result<poise::CreateReply, Error> {
    let mut builder = poise::CreateReply::default();
    if let Some(content) = message_payload.content {
        builder = builder.content(content);
    }
    if let Some(embeds) = message_payload.embeds {
        for embed in embeds {
            builder = builder.embed(build_embed(embed));
        }
    }
    if let Some(components) = message_payload.components {
        builder = builder.components(build_action_rows(components)?);
    }
    if let Some(ephemeral) = message_payload.ephemeral {
        builder = builder.ephemeral(ephemeral);
    }
    Ok(builder)
}

fn build_message_payload(
    message_payload: MessagePayload,
) -> Result<serenity::CreateMessage, Error> {
    if message_payload.ephemeral == Some(true) {
        bail!("ephemeral is only supported for interaction responses");
    }
    let mut message = serenity::CreateMessage::new();
    if let Some(content) = message_payload.content {
        message = message.content(content);
    }
    if let Some(embeds) = message_payload.embeds {
        for embed in embeds {
            message = message.add_embed(build_embed(embed));
        }
    }
    if let Some(components) = message_payload.components {
        message = message.components(build_action_rows(components)?);
    }
    Ok(message)
}

fn build_embed(embed_payload: EmbedPayload) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new();
    if let Some(title) = embed_payload.title {
        embed = embed.title(title);
    }
    if let Some(description) = embed_payload.description {
        embed = embed.description(description);
    }
    for field in embed_payload.fields {
        embed = embed.field(field.name, field.value, field.inline);
    }
    embed
}

fn build_action_rows(rows: Vec<ActionRowPayload>) -> Result<Vec<serenity::CreateActionRow>, Error> {
    rows.into_iter()
        .map(|row| match row {
            ActionRowPayload::Buttons(buttons) => {
                let buttons = buttons
                    .into_iter()
                    .map(build_button)
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(serenity::CreateActionRow::Buttons(buttons))
            }
            ActionRowPayload::SelectMenu(menu) => {
                build_select_menu(menu).map(serenity::CreateActionRow::SelectMenu)
            }
            ActionRowPayload::InputText(input) => Ok(serenity::CreateActionRow::InputText(
                build_input_text(input),
            )),
        })
        .collect()
}

fn build_button(spec: ButtonPayload) -> Result<serenity::CreateButton, Error> {
    let mut button = match spec.style {
        ButtonStylePayload::Link => {
            let url = spec
                .url
                .ok_or_else(|| anyhow!("link button requires url"))?;
            serenity::CreateButton::new_link(url)
        }
        _ => {
            let custom_id = spec
                .custom_id
                .ok_or_else(|| anyhow!("button requires custom_id"))?;
            serenity::CreateButton::new(custom_id).style(map_button_style(&spec.style))
        }
    };

    if let Some(label) = spec.label {
        button = button.label(label);
    }
    if spec.disabled {
        button = button.disabled(true);
    }
    Ok(button)
}

fn map_button_style(style: &ButtonStylePayload) -> serenity::ButtonStyle {
    match style {
        ButtonStylePayload::Primary => serenity::ButtonStyle::Primary,
        ButtonStylePayload::Secondary => serenity::ButtonStyle::Secondary,
        ButtonStylePayload::Success => serenity::ButtonStyle::Success,
        ButtonStylePayload::Danger => serenity::ButtonStyle::Danger,
        ButtonStylePayload::Link => unreachable!("link buttons are handled separately"),
    }
}

fn build_select_menu(menu: SelectMenuPayload) -> Result<serenity::CreateSelectMenu, Error> {
    let options = menu
        .options
        .into_iter()
        .map(build_select_option)
        .collect::<Vec<_>>();
    let kind = serenity::CreateSelectMenuKind::String { options };
    let mut select = serenity::CreateSelectMenu::new(menu.custom_id, kind);
    if let Some(placeholder) = menu.placeholder {
        select = select.placeholder(placeholder);
    }
    if let Some(min) = menu.min_values {
        select = select.min_values(min);
    }
    if let Some(max) = menu.max_values {
        select = select.max_values(max);
    }
    if menu.disabled {
        select = select.disabled(true);
    }
    Ok(select)
}

fn build_select_option(option: SelectOptionPayload) -> serenity::CreateSelectMenuOption {
    let mut opt = serenity::CreateSelectMenuOption::new(option.label, option.value);
    if let Some(description) = option.description {
        opt = opt.description(description);
    }
    if option.default {
        opt = opt.default_selection(true);
    }
    opt
}

fn build_modal(spec: ModalPayload) -> Result<serenity::CreateModal, Error> {
    let rows = spec
        .inputs
        .into_iter()
        .map(build_input_text)
        .map(serenity::CreateActionRow::InputText)
        .collect();
    Ok(serenity::CreateModal::new(spec.custom_id, spec.title).components(rows))
}

fn build_input_text(spec: TextInputPayload) -> serenity::CreateInputText {
    let mut input =
        serenity::CreateInputText::new(map_input_style(spec.style), spec.label, spec.custom_id);
    if let Some(placeholder) = spec.placeholder {
        input = input.placeholder(placeholder);
    }
    if let Some(min) = spec.min_length {
        input = input.min_length(min);
    }
    if let Some(max) = spec.max_length {
        input = input.max_length(max);
    }
    if let Some(value) = spec.value {
        input = input.value(value);
    }
    input.required(spec.required)
}

fn map_input_style(style: TextInputStylePayload) -> serenity::InputTextStyle {
    match style {
        TextInputStylePayload::Short => serenity::InputTextStyle::Short,
        TextInputStylePayload::Paragraph => serenity::InputTextStyle::Paragraph,
    }
}
