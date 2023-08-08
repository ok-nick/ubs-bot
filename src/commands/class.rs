use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    futures::TryStreamExt,
    model::prelude::*,
    prelude::*,
};
use ubs_lib::{model::ClassModel, parser::ClassSchedule};

const TIME_FORMAT: &str = "%-I:%M%p";
const UNKNOWN_FIELD: &str = "[unknown]";

#[command]
#[sub_commands(info, watch, unwatch)]
pub async fn class(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "Please specify a subcommand.").await?;
    Ok(())
}

#[command]
#[description("Get information for class.")]
pub async fn info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let course = args.single::<String>()?.to_uppercase();
    let semester = args.single::<String>()?.to_uppercase();
    let section = args.single::<String>()?.to_uppercase();

    match fetch_class_info(&course, &semester, &section).await? {
        Some(class) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        // TODO: a lot of this is super duper ugly and boilerplate
                        e.title(format!("{} - {}", course, semester))
                            .author(|a| {
                                a.name(class.instructor.as_deref().unwrap_or(UNKNOWN_FIELD))
                            })
                            // .field("Id", class.class_id.unwrap(), true)
                            .field(
                                "Section",
                                class.section.as_deref().unwrap_or(UNKNOWN_FIELD),
                                true,
                            )
                            .field(
                                "Type",
                                class
                                    .class_type
                                    .map(|x| x.to_string())
                                    .as_deref()
                                    .unwrap_or(UNKNOWN_FIELD),
                                true,
                            )
                            .field("Room", class.room.as_deref().unwrap_or(UNKNOWN_FIELD), true)
                            .field(
                                "Open",
                                class
                                    .is_open
                                    .map(|x| x.to_string())
                                    .as_deref()
                                    .unwrap_or(UNKNOWN_FIELD),
                                true,
                            )
                            .field(
                                "Seats",
                                format!(
                                    "{}/{}",
                                    class
                                        .open_seats
                                        .map(|x| x.to_string())
                                        .as_deref()
                                        .unwrap_or(UNKNOWN_FIELD),
                                    class
                                        .total_seats
                                        .map(|x| x.to_string())
                                        .as_deref()
                                        .unwrap_or(UNKNOWN_FIELD),
                                ),
                                true,
                            )
                            .field(
                                // TODO: dynamically set plurality
                                "Day(s) of Week",
                                match class.days_of_week {
                                    Some(dow) => dow
                                        .into_iter()
                                        .map(|x| {
                                            x.map(|y| y.to_string())
                                                .unwrap_or(UNKNOWN_FIELD.to_owned())
                                        })
                                        .collect::<Vec<String>>()
                                        .join(", "),
                                    None => UNKNOWN_FIELD.to_owned(),
                                },
                                false,
                            )
                            .field(
                                "Time",
                                format!(
                                    "{} — {}",
                                    class
                                        .start_time
                                        .map(|x| x.format(TIME_FORMAT).to_string())
                                        .as_deref()
                                        .unwrap_or(UNKNOWN_FIELD),
                                    class
                                        .end_time
                                        .map(|x| x.format(TIME_FORMAT).to_string())
                                        .as_deref()
                                        .unwrap_or(UNKNOWN_FIELD),
                                ),
                                true,
                            )
                    })
                })
                .await?;
        }
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Could not find {course}, section {section}, during {semester}."),
                )
                .await?;
        }
    }

    typing.stop();

    Ok(())
}

// TODO:
//    [prefix]class watch [course] [semester] [section]
//    [prefix]class watch cse115 spring2023 a5
#[command]
#[description("Notify when class opens.")]
pub async fn watch(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    typing.stop();
    todo!()
}

// TODO:
//    [prefix]class unwatch [course] [semester] [section]
//    [prefix]class unwatch cse115 spring2023 a5
#[command]
#[description("Stop notifying when class opens.")]
pub async fn unwatch(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    typing.stop();
    todo!()
}

// TODO: cache schedules
async fn fetch_class_info(
    course: &str,
    semester: &str,
    section: &str,
) -> CommandResult<Option<ClassModel>> {
    let mut schedule_iter = ubs_lib::schedule_iter(course.parse()?, semester.parse()?).await?;

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        let schedule = schedule?;
        if let Some(class) = class_from_schedule(section, &schedule)? {
            return Ok(Some(class));
        }

        break;
    }

    Ok(None)
}

fn class_from_schedule(
    section: &str,
    schedule: &ClassSchedule,
) -> CommandResult<Option<ClassModel>> {
    for group in schedule.group_iter() {
        for class in group.class_iter() {
            if class.section()? == section {
                return Ok(Some(class.model()?));
            }
        }
    }

    Ok(None)
}
