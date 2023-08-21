mod cache;
mod commands;
mod watcher;

use std::{env, time::Duration};

use cache::Cache;
use poise::{serenity_prelude::GatewayIntents, Framework, FrameworkOptions};

use sqlx::PgPool;
use tracing::error;

const WATCHER_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
const STALE_DATA_DURATION: Duration = Duration::from_secs(1);

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    cache: Cache,
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("missing `DISCORD_TOKEN` environment variable");

    // let pool = PgPool::connect(&env::var("DATABASE_URL").unwrap())
    let database = PgPool::connect("postgres://postgres:password@localhost/ubs")
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("failed to migrate database");

    let cache = Cache::new(database, STALE_DATA_DURATION);
    let framework = Framework::builder()
        .token(token)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT)
        .options(FrameworkOptions {
            commands: vec![
                commands::info(),
                commands::rawinfo(),
                commands::watch(),
                commands::unwatch(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { cache })
            })
        })
        .build()
        .await
        .expect("TODO");

    {
        let framework = framework.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Could not register ctrl+c handler");
            framework.shard_manager().lock().await.shutdown_all().await;
        });
    }

    if let Err(why) = framework.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
