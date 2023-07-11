mod commands;
mod handlers;
mod notes;
mod session;
mod utils;

use anyhow::Context as _;
use poise::{
    serenity_prelude as serenity, serenity_prelude::GatewayIntents, Event, Framework,
    FrameworkOptions,
};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use surrealdb::{engine::remote::ws, opt::auth::Root, Surreal};

type SurrealEngine = ws::Client;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
pub struct Data {
    pub database: Surreal<SurrealEngine>,
}

async fn event_handler(_: &serenity::Context, event: &Event<'_>, data: &Data) -> Result<(), Error> {
    let db = &data.database;
    if let Event::Message { new_message } = event {
        match new_message.kind {
            serenity::MessageType::Regular => handlers::handle_new_message(new_message, db).await?,
            _ => (),
        }
    };

    Ok(())
}

#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    let intents: GatewayIntents =
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' not found in environment")?;
    let commands = vec![
        commands::start_session(),
        commands::get_sessions(),
        commands::end_session(),
        commands::get_note(),
    ];
    let options = FrameworkOptions {
        commands,
        
        event_handler: |ctx, event, _, data| Box::pin(event_handler(ctx, event, data)),
        ..Default::default()
    };

    let surreal_url = secret_store
        .get("SURREAL_DB_URL")
        .context("'SURREAL_DB_URL' not found in environment")?;
    let db = Surreal::new::<ws::Ws>(surreal_url).await.unwrap();
    let username = secret_store.get("SURREAL_DB_USERNAME").unwrap_or("root".to_string());
    let password = secret_store.get("SURREAL_DB_PASSWORD").unwrap_or("root".to_string());
    db.signin(Root {
        username: &username,
        password: &password,
    })
    .await
    .unwrap();
    let f = Framework::builder()
        .options(options)
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { database: db })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(f.into())
}
