use poise::serenity_prelude::{MessageId, UserId};
use surrealdb::{sql, Connection, Surreal};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionEntry {
    id: sql::Thing,
    pub content: String,
    pub author: u64,
    pub session: sql::Thing,
}

pub async fn create_entry<T: Connection>(
    content: &String,
    message_id: MessageId,
    author_id: UserId,
    session_id: &sql::Thing,
    db: &Surreal<T>,
) -> Result<sql::Thing, surrealdb::Error> {
    let entry = SessionEntry {
        id: sql::Thing::from(("session_entry".into(), sql::Id::from(message_id.0))),
        content: content.clone(),
        author: author_id.0,
        session: session_id.clone(),
    };

    let created: SessionEntry = db.create("session_entry").content(entry).await?;

    Ok(created.id)
}

pub async fn get_for_session<T: Connection>(session_id: &sql::Thing, db: &Surreal<T>)-> Result<Vec<SessionEntry>, surrealdb::Error> {
    let mut result = db
        .query("SELECT * from session_entry WHERE session = $s;")
        .bind(("s", session_id)).await?;

    let notes: Vec<SessionEntry> = result.take(0)?;

    Ok(notes)
}
