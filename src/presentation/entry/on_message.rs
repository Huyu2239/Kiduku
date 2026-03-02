use std::collections::{HashMap, HashSet};

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::{ChannelType, Member, RoleId, UserId};

use crate::infrastructure::db::NewMention;
use crate::interface::mapper::input_mapper;
use crate::presentation::entry::on_error;
use crate::presentation::entry::util::{
    DONE_EMOJI_ID, DONE_EMOJI_NAME, KIDOKU_EMOJI_ID, KIDOKU_EMOJI_NAME,
};
use crate::presentation::{Data, Error};
use crate::usecase::on_message::auto_add_read_reaction;

pub async fn handle(ctx: &serenity::Context, data: &Data, message: &serenity::Message) {
    if message.author.bot {
        return;
    }

    match message.kind {
        serenity::MessageType::Regular | serenity::MessageType::InlineReply => {}
        _ => return,
    }

    let input = input_mapper::from_message_to_message_input_dto(message);
    let output = match auto_add_read_reaction::execute(input) {
        Ok(output) => output,
        Err(err) => {
            tracing::error!("usecase error: {:?}", err);
            return;
        }
    };

    if !output.should_add_reaction {
        return;
    }

    let guild_id = match message.guild_id {
        Some(guild_id) => guild_id,
        None => {
            tracing::warn!("message without guild_id cannot be stored");
            return;
        }
    };

    let bot_id = ctx.cache.current_user().id;
    let targets = match collect_targets(ctx, guild_id, message, bot_id).await {
        Ok(targets) => targets,
        Err(err) => {
            on_error::handle_exec_error(err);
            Vec::new()
        }
    };

    let mention = NewMention {
        guild_id: guild_id.get(),
        channel_id: message.channel_id.get(),
        message_id: message.id.get(),
        author_id: message.author.id.get(),
        content: message.content.clone(),
        mention_everyone: message.mention_everyone,
        created_at_unix: message.timestamp.unix_timestamp(),
        targets,
    };

    if mention.targets.is_empty() {
        tracing::warn!("mention detected but targets were empty; skipping DB insert");
    } else if let Err(err) = data.db.insert_mention(mention).await {
        on_error::handle_exec_error(err);
    }

    let kidoku_reaction = serenity::ReactionType::Custom {
        animated: false,
        id: serenity::EmojiId::new(KIDOKU_EMOJI_ID),
        name: Some(KIDOKU_EMOJI_NAME.to_string()),
    };
    if let Err(err) = message.react(&ctx.http, kidoku_reaction).await {
        on_error::handle_exec_error(err.into());
    }

    let done_reaction = serenity::ReactionType::Custom {
        animated: false,
        id: serenity::EmojiId::new(DONE_EMOJI_ID),
        name: Some(DONE_EMOJI_NAME.to_string()),
    };
    if let Err(err) = message.react(&ctx.http, done_reaction).await {
        on_error::handle_exec_error(err.into());
    }
}

async fn collect_targets(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    message: &serenity::Message,
    bot_id: UserId,
) -> Result<Vec<u64>, Error> {
    if !has_mentions(message) {
        return Ok(Vec::new());
    }

    let members = fetch_guild_members(ctx, guild_id).await?;
    let role_members = build_role_members_map(&members);
    let everyone_members = resolve_everyone_targets(ctx, message, &members, bot_id).await;

    let mut targets = Vec::new();
    targets.extend(
        message
            .mentions
            .iter()
            .filter(|user| !user.bot && user.id != bot_id)
            .map(|user| user.id.get()),
    );
    targets.extend(
        expand_role_members(&role_members, &message.mention_roles)
            .into_iter()
            .filter(|user_id| *user_id != bot_id)
            .map(|user_id| user_id.get()),
    );
    if message.mention_everyone {
        targets.extend(everyone_members.into_iter().map(|user_id| user_id.get()));
    }

    targets.sort_unstable();
    targets.dedup();
    Ok(targets)
}

fn has_mentions(message: &serenity::Message) -> bool {
    !message.mentions.is_empty() || !message.mention_roles.is_empty() || message.mention_everyone
}

async fn fetch_guild_members(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
) -> Result<Vec<Member>, Error> {
    let members = guild_id
        .members(&ctx.http, Some(1000), None)
        .await
        .context("failed to fetch guild members. enable the GUILD_MEMBERS intent")?;
    Ok(members)
}

async fn resolve_everyone_targets(
    ctx: &serenity::Context,
    message: &serenity::Message,
    members: &[Member],
    bot_id: UserId,
) -> Vec<UserId> {
    let guild_targets = members
        .iter()
        .filter(|member| !member.user.bot && member.user.id != bot_id)
        .map(|member| member.user.id)
        .collect::<Vec<_>>();

    if !message.mention_everyone {
        return Vec::new();
    }

    if !is_thread_message(ctx, message).await {
        return guild_targets;
    }

    let thread_members = match message.channel_id.get_thread_members(&ctx.http).await {
        Ok(members) => members,
        Err(err) => {
            tracing::warn!(
                "failed to fetch thread members for @everyone/@here, fallback to guild targets: {:?}",
                err
            );
            return guild_targets;
        }
    };

    let non_bot_user_ids = members
        .iter()
        .filter(|member| !member.user.bot)
        .map(|member| member.user.id)
        .collect::<HashSet<_>>();

    let mut targets = thread_members
        .iter()
        .map(|member| member.user_id)
        .filter(|user_id| *user_id != bot_id)
        .filter(|user_id| non_bot_user_ids.contains(user_id))
        .collect::<Vec<_>>();
    targets.sort_unstable();
    targets.dedup();
    targets
}

async fn is_thread_message(ctx: &serenity::Context, message: &serenity::Message) -> bool {
    match message.channel_id.to_channel(&ctx.http).await {
        Ok(serenity::Channel::Guild(channel)) => matches!(
            channel.kind,
            ChannelType::PublicThread | ChannelType::PrivateThread | ChannelType::NewsThread
        ),
        _ => false,
    }
}

fn build_role_members_map(members: &[Member]) -> HashMap<RoleId, Vec<UserId>> {
    let mut map: HashMap<RoleId, Vec<UserId>> = HashMap::new();
    for member in members {
        if member.user.bot {
            continue;
        }
        for role_id in &member.roles {
            map.entry(*role_id).or_default().push(member.user.id);
        }
    }
    map
}

fn expand_role_members(
    role_members: &HashMap<RoleId, Vec<UserId>>,
    role_mentions: &[RoleId],
) -> Vec<UserId> {
    let mut expanded = Vec::new();
    for role_id in role_mentions {
        if let Some(users) = role_members.get(role_id) {
            expanded.extend(users.iter().copied());
        }
    }
    expanded
}
