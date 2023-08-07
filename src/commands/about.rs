use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
pub async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    // TODO: include github url
    msg.reply(
        &ctx.http,
        "I manage queries for University at Buffalo course schedules.",
    )
    .await?;

    Ok(())
}
