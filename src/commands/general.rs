// use std::collections::HashSet;

// use serenity::{
//     framework::standard::{
//         help_commands,
//         macros::{command, help},
//         Args, CommandGroup, CommandResult, HelpOptions,
//     },
//     model::prelude::*,
//     prelude::*,
// };

// #[help]
// async fn help(
//     context: &Context,
//     msg: &Message,
//     args: Args,
//     help_options: &'static HelpOptions,
//     groups: &[&'static CommandGroup],
//     owners: HashSet<UserId>,
// ) -> CommandResult {
//     let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
//     Ok(())
// }

// #[command]
// pub async fn about(ctx: &Context, msg: &Message) -> CommandResult {
//     // TODO: include github url
//     msg.reply(
//         &ctx.http,
//         "I manage queries for University at Buffalo course schedules.",
//     )
//     .await?;

//     Ok(())
// }
