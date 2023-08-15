mod cache;
mod commands;
mod watcher;

use std::{env, time::Duration};

use serenity::{
    async_trait,
    framework::{standard::macros::group, StandardFramework},
    model::gateway::Ready,
    prelude::*,
};

use commands::*;
use sqlx::PgPool;
use tracing::{error, info};

const WATCHER_UPDATE_INTERVAL: Duration = Duration::from_secs(1);

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(about)]
struct General;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("missing `DISCORD_TOKEN` environment variable");

    let pool = PgPool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to migrate database");

    let framework = StandardFramework::new()
        .configure(|c| c.allow_dm(false))
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&CLASS_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .type_map_insert::<Pool>(pool.clone())
        .await
        .expect("failed to create client");

    // tokio::spawn(watcher_loop(pool));

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("client error: {:?}", why);
    }
}
