mod commands;

use std::{env, time::Duration};

use serenity::{
    async_trait,
    framework::{standard::macros::group, StandardFramework},
    model::gateway::Ready,
    prelude::*,
};

use commands::*;
use sqlx::{Database, PgPool};
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

    tokio::spawn(watcher_loop(pool));

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

async fn watcher_loop(pool: PgPool) {
    loop {
        let queries = sqlx::query!(
            r#"
SELECT course, semester, career, section
FROM watchers
GROUP BY (course, semester, career, section);
                "#
        )
        .fetch_all(&pool)
        .await
        .unwrap(); // TODO: handle

        // TODO: check query for updates (need cache)
        //       if update then notify users

        for query_rec in queries {
            println!("{:?}", query_rec);
            let user_ids = sqlx::query!(
                r#"
SELECT user_id
FROM watchers
WHERE
  $1 in (course)
  AND
  $2 in (semester)
  AND
  $3 in (career)
  AND
  $4 in (section);
                "#,
                query_rec.course,
                query_rec.semester,
                query_rec.career,
                query_rec.section,
            )
            .fetch_all(&pool)
            .await
            .unwrap(); // TODO: handle
            for user_id_rec in user_ids {
                println!("{:?}", user_id_rec.user_id);
            }
        }

        tokio::time::sleep(WATCHER_UPDATE_INTERVAL).await;
    }
}
