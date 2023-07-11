use std::fmt::Write;

use ::serde::{Deserialize, Serialize};
use chrono::*;
use surrealdb::{error::Api, sql, Connection, Error as SurrealError, Surreal};

use crate::Error;

type DateTime = chrono::DateTime<Utc>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]

pub struct ScriberSession {
    pub id: sql::Thing,
    pub created_at: DateTime,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelState {
    id: sql::Thing,
    active_session: ScriberSession,
}

impl ScriberSession {
    fn new(name: &str) -> Self {
        ScriberSession {
            id: sql::Thing::from(("session".into(), sql::Id::ulid())),
            created_at: Utc::now(),
            name: name.to_string(),
        }
    }

    fn from_id(id: i64, name: &str) -> Self {
        ScriberSession {
            id: sql::Thing::from(("session".into(), sql::Id::from(id))),
            created_at: Utc::now(),
            name: name.to_string(),
        }
    }
}

pub async fn get_all<T: Connection>(db: &Surreal<T>) -> Result<Vec<ScriberSession>, Error> {
    let sessions: Vec<ScriberSession> = db.select("session").await?;

    Ok(sessions)
}

pub async fn create<T: Connection>(
    id: Option<i64>,
    name: &str,
    channel_id: i64,
    db: &Surreal<T>,
) -> Result<sql::Thing, SurrealError> {
    let session = if id.is_some() {
        ScriberSession::from_id(id.unwrap(), name)
    } else {
        ScriberSession::new(name)
    };

    let data = ChannelState {
        id: sql::Thing::from(("channel_state".into(), sql::Id::from(channel_id))),
        active_session: session.clone(),
    };
    let session_query = "CREATE type::thing('session', $session_id) CONTENT $session_content";
    let channel_query = "CREATE type::thing('channel_state', $channel_id) CONTENT $channel_state";
    let queries = [session_query, channel_query];
    let mut response = db
        .query(format!(
            "BEGIN TRANSACTION;\n{};\nCOMMIT TRANSACTION;",
            queries.join(";\n")
        ))
        .bind(("channel_id", channel_id))
        .bind(("session_id", &session.id))
        .bind(("channel_state", data))
        .bind(("session_content", session.clone()))
        .await?;
    println!("{:?}", response);
    let mut errors = response.take_errors();
    if errors.len() != 0 {
        let message = errors.drain().fold(String::new(), |mut acc, current| {
            writeln!(acc, "{current:?}").unwrap();
            acc
        });
        return Err(SurrealError::Api(Api::Query(message)));
    }

    return Ok(session.id);
}

pub async fn end<T: Connection>(
    channel_id: i64,
    db: &Surreal<T>,
) -> Result<Option<sql::Thing>, Error> {
    let active_session = get_channel_active_session(channel_id, db).await?;

    if let Some(active_session) = active_session {
        let resource = sql::Thing::from(("channel_state".into(), sql::Id::from(channel_id)));
        let _: ChannelState = db.delete(resource).await?;

        return Ok(Some(active_session));
    }

    Ok(None)
}

pub async fn get_channel_active_session<T: Connection>(
    channel_id: i64,
    db: &Surreal<T>,
) -> Result<Option<sql::Thing>, Error> {
    let resource = sql::Thing::from(("channel_state".into(), sql::Id::from(channel_id)));
    let channel_state: Option<ChannelState> = db.select(resource).await?;
    if let Some(state) = channel_state {
        return Ok(Some(state.active_session.id));
    };

    Ok(None)
}
