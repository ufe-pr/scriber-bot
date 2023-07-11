use poise::serenity_prelude as serenity;
use surrealdb::Surreal;

use crate::{notes, session, Error, SurrealEngine};

pub async fn handle_new_message(
    message: &serenity::Message,
    db: &Surreal<SurrealEngine>,
) -> Result<(), Error> {
    db.use_ns("scriber_notes")
        .use_db(message.guild_id.unwrap_or(serenity::GuildId(0)).to_string())
        .await?;
    // If session is currently active in channel and not a bot message
    if !message.author.bot {
        if let Some(active_session) =
            session::get_channel_active_session(message.channel_id.into(), db).await?
        {
            let author = message.author.id;
            let content = &message.content;
            let message_id = message.id;
            notes::create_entry(content, message_id, author, &active_session, db).await?;
            // TODO: Consider deleting message
        }
    };

    Ok(())
}
