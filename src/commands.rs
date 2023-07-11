use std::time::Duration;

use crate::{notes, session as sessions_lib, utils, Context, Error, SurrealEngine};
use poise::serenity_prelude as serenity;
use surrealdb::Surreal;
use tokio::{io::AsyncWriteExt, fs};

const NO_SESSIONS_RESPONSE: &str = "No sessions here yet. Get started by calling /start_session.";

#[poise::command(slash_command, guild_only)]
pub async fn start_session(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let db = &ctx.data().database;
    db.use_ns("scriber_notes")
        .use_db(ctx.guild_id().unwrap_or(serenity::GuildId(0)).to_string())
        .await?;

    if let Err(e) =
        sessions_lib::create(Some(ctx.id() as i64), &name, ctx.channel_id().into(), db).await
    {
        if let surrealdb::Error::Api(message) = e.into() {
            if let surrealdb::error::Api::Query(message) = message {
                if message.contains("already exists") {
                    ctx.say("There can be only one active session in a channel at a time!")
                        .await?;
                }
            }
        };

        return Ok(());
    };
    ctx.say("Session started!").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn get_sessions(ctx: Context<'_>) -> Result<(), Error> {
    let db = get_db_for_context(ctx).await?;
    let sessions = sessions_lib::get_all(db).await?;

    if sessions.len() == 0 {
        ctx.say(NO_SESSIONS_RESPONSE)
            .await?;
        return Ok(());
    }

    let mut response = String::from("Here's all your sessions:\n\n");
    for session in sessions {
        let form = format!(
            "* {} (<t:{}>)\n",
            session.name,
            session.created_at.timestamp()
        );
        response.push_str(&form);
    }
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn end_session(ctx: Context<'_>) -> Result<(), Error> {
    let db = get_db_for_context(ctx).await?;
    let response = match sessions_lib::end(ctx.channel_id().into(), db).await? {
        Some(_) => "Session ended!",
        None => "No active session in this channel!",
    };

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn get_note(ctx: Context<'_>) -> Result<(), Error> {
    let db = get_db_for_context(ctx).await?;
    let sessions = sessions_lib::get_all(db).await?;
    const OUTPUT_LIMIT: u16 = 500;

    if sessions.len() == 0 {
        ctx.say(NO_SESSIONS_RESPONSE)
            .await?;
        return Ok(());
    }

    let result = ctx
        .send(|f| {
            f.ephemeral(true);
            f.components(|f| {
                f.create_action_row(|r| {
                    r.create_select_menu(|menu| {
                        menu.custom_id("select_session")
                            .placeholder("Select a session");
                        menu.options(|f| {
                            for s in &sessions {
                                f.create_option(|o| {
                                    o.label(&s.name).value(&s.id).description(format!(
                                        "Started {}",
                                        s.created_at.format("%d/%m/%Y %H:%M UTC%:::z")
                                    ))
                                });
                            }
                            f
                        })
                    })
                })
            })
        })
        .await?;

    let message = result.message().await?;
    let interaction = match message
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(x) => x,
        None => {
            return Ok(());
        }
    };

    let session_id = &interaction.data.values[0];
    let session_id = surrealdb::sql::thing(&session_id)?;
    let no_session = &"No session selected".to_string();
    let session_name = sessions
        .iter()
        .find(|&x| x.id == session_id)
        .map(|x| &x.name)
        .unwrap_or(no_session);

    let all_notes = notes::get_for_session(&session_id, db).await?;
    let output = utils::build_session_notes_output(&all_notes)?;

    let path = format!("{}.txt", ctx.id());
    let mut file = fs::File::create(&path).await?;

    if output.len() > OUTPUT_LIMIT.into() {
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .await?
            .write_all(output.as_bytes())
            .await?;

        file = fs::OpenOptions::new().read(true).open(&path).await?;
    };

    // TODO: Consider saving a reference to this response and respond with it if same session is requested later on
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(serenity::InteractionResponseType::UpdateMessage);
            r.interaction_response_data(|d| {
                d.ephemeral(false);
                d.components(|f| f);
                if output.is_empty() {
                    d.content(&format!("**{}**\n\nSession has no content", session_name))
                } else if output.len() <= OUTPUT_LIMIT.into() {
                    d.content(format!("**{session_name}**\n\n{output}"))
                } else {
                    let file_name = format!("{} notes.txt", session_name);
                    let file_name = file_name.as_str();

                    d.add_file((&file, file_name));
                    d
                }
            })
        })
        .await?;

    // Remove the file after sending it
    fs::remove_file(path).await?;

    Ok(())
}

async fn get_db_for_context(ctx: Context<'_>) -> Result<&Surreal<SurrealEngine>, Error> {
    let db = &ctx.data().database;
    db.use_ns("scriber_notes")
        .use_db(ctx.guild_id().unwrap_or(serenity::GuildId(0)).to_string())
        .await?;
    Ok(db)
}
