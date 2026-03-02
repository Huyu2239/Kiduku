use std::time::Duration;

use poise::serenity_prelude as serenity;

use crate::infrastructure::db::Db;

pub mod discord_exec;
pub mod entry;

#[derive(Clone)]
pub struct Data {
    pub db: Db,
}

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub fn build_framework(db: Db) -> poise::Framework<Data, Error> {
    let options = poise::FrameworkOptions {
        commands: entry::slash_commands::all(),
        on_error: |error| Box::pin(entry::on_error::handle_framework_error(error)),
        event_handler: |ctx, event, _framework, data| {
            Box::pin(async move {
                handle_event(ctx, event, data).await;
                Ok(())
            })
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .options(options)
        .setup(move |ctx, ready, framework| {
            let db = db.clone();
            Box::pin(async move {
                tracing::info!("logged in as {}", ready.user.name);
                let commands =
                    poise::builtins::create_application_commands(&framework.options().commands);
                let ctx_clone = ctx.clone();
                tokio::spawn(async move {
                    register_commands_with_retry(ctx_clone, commands).await;
                });
                let db_for_batch = db.clone();
                let ctx_for_batch = ctx.clone();
                entry::batch::start(ctx_for_batch, db_for_batch);
                Ok(Data { db })
            })
        })
        .build()
}

async fn handle_event(ctx: &serenity::Context, event: &serenity::FullEvent, data: &Data) {
    if let serenity::FullEvent::Message { new_message } = event {
        entry::on_message::handle(ctx, data, new_message).await;
        return;
    }

    if let serenity::FullEvent::ReactionAdd { add_reaction } = event {
        entry::on_reaction_add::handle(ctx, data, add_reaction).await;
        return;
    }

    if let serenity::FullEvent::MessageDelete {
        deleted_message_id, ..
    } = event
    {
        entry::on_message_delete::handle_single(data, *deleted_message_id).await;
        return;
    }

    if let serenity::FullEvent::MessageDeleteBulk {
        multiple_deleted_messages_ids,
        ..
    } = event
    {
        entry::on_message_delete::handle_bulk(data, multiple_deleted_messages_ids).await;
        return;
    }

    if let serenity::FullEvent::InteractionCreate {
        interaction: serenity::Interaction::Component(comp),
    } = event
    {
        entry::on_component::handle(ctx, data, comp).await;
    }
}

async fn register_commands_with_retry(
    ctx: serenity::Context,
    commands: Vec<serenity::CreateCommand>,
) {
    const MAX_ATTEMPTS: usize = 3;
    let mut delay = Duration::from_secs(2);

    for attempt in 1..=MAX_ATTEMPTS {
        match serenity::Command::set_global_commands(&ctx.http, commands.clone()).await {
            Ok(registered) => {
                tracing::info!("registered {} commands globally", registered.len());
                return;
            }
            Err(err) => {
                let retry_plan = retry_plan_for_register_error(&err);
                if !retry_plan.is_retryable() {
                    tracing::error!(
                        "failed to register global commands (non-retryable): {:?}",
                        err
                    );
                    return;
                }

                if attempt == MAX_ATTEMPTS {
                    tracing::error!(
                        "failed to register global commands after {} attempts: {:?}",
                        attempt,
                        err
                    );
                    return;
                }

                tracing::warn!(
                    "failed to register global commands (attempt {}/{}). retrying in {:?}: {:?}",
                    attempt,
                    MAX_ATTEMPTS,
                    delay,
                    err
                );
                tokio::time::sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(30));
            }
        }
    }
}

fn retry_plan_for_register_error(err: &serenity::Error) -> RetryPlan {
    match err {
        serenity::Error::Http(http_err) => retry_plan_for_http_error(http_err),
        serenity::Error::Io(_) => RetryPlan::retry(),
        _ => RetryPlan::non_retryable(),
    }
}

fn retry_plan_for_http_error(err: &serenity::http::HttpError) -> RetryPlan {
    match err {
        serenity::http::HttpError::UnsuccessfulRequest(response) => {
            let status = response.status_code.as_u16();
            if matches!(status, 408 | 500 | 502 | 503 | 504) {
                RetryPlan::retry()
            } else {
                RetryPlan::non_retryable()
            }
        }
        serenity::http::HttpError::Request(req_err) => {
            if req_err.is_timeout() || req_err.is_connect() {
                RetryPlan::retry()
            } else {
                RetryPlan::non_retryable()
            }
        }
        serenity::http::HttpError::RateLimitI64F64 | serenity::http::HttpError::RateLimitUtf8 => {
            RetryPlan::retry()
        }
        serenity::http::HttpError::Url(_)
        | serenity::http::HttpError::InvalidWebhook
        | serenity::http::HttpError::InvalidHeader(_)
        | serenity::http::HttpError::InvalidScheme
        | serenity::http::HttpError::InvalidPort
        | serenity::http::HttpError::ApplicationIdMissing => RetryPlan::non_retryable(),
        _ => RetryPlan::non_retryable(),
    }
}

#[derive(Clone, Copy)]
struct RetryPlan {
    retryable: bool,
}

impl RetryPlan {
    fn retry() -> Self {
        Self { retryable: true }
    }

    fn non_retryable() -> Self {
        Self { retryable: false }
    }

    fn is_retryable(self) -> bool {
        self.retryable
    }
}
