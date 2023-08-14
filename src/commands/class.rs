use serenity::{
    builder::CreateMessage,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    futures::TryStreamExt,
    model::prelude::*,
    prelude::*,
};
use sqlx::PgPool;
use ubs_lib::{model::ClassModel, parser::ClassSchedule, Course, Semester};

const TIME_FORMAT: &str = "%-I:%M%p";
const UNKNOWN_FIELD: &str = "[unknown]";

pub struct Pool;

impl TypeMapKey for Pool {
    type Value = PgPool;
}

#[group]
#[prefixes("class")]
#[summary = "Commands for querying class schedules"]
#[commands(info, watch, unwatch)]
struct Class;

#[command]
#[sub_commands(info_raw)]
#[description("Get information of class")]
pub async fn info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let course = args.single::<String>()?.to_uppercase();
    let semester = args.single::<String>()?.to_uppercase();
    let section = args.single::<String>()?.to_uppercase();

    match fetch_class_info(course.parse()?, semester.parse()?, &section).await? {
        Some(class) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    create_class_info_message(m, &class, &course, &semester)
                })
                .await?;
        }
        None => {
            // TODO: use embeds
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Could not find {course}, section {section}, during {semester}.

*Does this class exist? It may just be missing a mapping. Read here for more information: https://github.com/ok-nick/ubs-bot#why-cant-it-find-a-class-that-i-know-exists*"),
                )
                .await?;
        }
    }

    typing.stop();
    Ok(())
}

// TODO: insane boilerplate
#[command]
#[aliases("raw")] // TODO: can I make it so raw is the only way to call it?
#[description("Get information of class using raw ids")]
pub async fn info_raw(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let course = args.single::<String>()?;
    let semester = args.single::<String>()?;
    // TODO: third parameter should be the career id
    let section = args.single::<String>()?;

    // TODO: in `ubs-lib` impl Display on ids to avoid clone
    match fetch_class_info(
        Course::Raw(course.clone()),
        Semester::Raw(semester.clone()),
        &section,
    )
    .await?
    {
        Some(class) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    create_class_info_message(m, &class, &course, &semester)
                })
                .await?;
        }
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Could not find course id {course}, section {section}, during semester id {semester}."),
                )
                .await?;
        }
    }

    typing.stop();
    Ok(())
}

#[command]
#[description("Notify when class opens")]
pub async fn watch(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    let course = args.single::<String>()?;
    let semester = args.single::<String>()?;
    let section = args.single::<String>()?;
    // TODO: if career doesn't exist, then infer it, if it can't infer, report to user
    let career = args.single::<String>()?;

    {
        let guard = ctx.data.read().await;
        let pool = guard.get::<Pool>().unwrap(); // TODO: fix unwrap
        sqlx::query!(
            "INSERT INTO watchers VALUES ($1, $2, $3, $4, $5)",
            msg.author.id.0 as i64,
            course,
            semester,
            career,
            section
        )
        .execute(pool)
        .await?;
    };

    typing.stop();
    Ok(())
}

#[command]
#[description("Stop notifying when class opens")]
pub async fn unwatch(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;

    // TODO: remove from queue

    typing.stop();
    todo!()
}

// TODO: absolute monstrosity of a function
fn create_class_info_message<'a, 'b>(
    m: &'a mut CreateMessage<'b>,
    class: &ClassModel,
    course: &str,
    semester: &str,
) -> &'a mut CreateMessage<'b> {
    m.embed(|e| {
        // TODO: set timestamp of embed to last time data was updated. so if the data was taken from a cache
        //       you know when it was last updated
        e.title(format!("{} - {}", course, semester))
            .author(|a| a.name(class.instructor.as_deref().unwrap_or(UNKNOWN_FIELD)))
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
                    Some(ref dow) => dow
                        .iter()
                        .map(|x| x.map(|y| y.to_string()).unwrap_or(UNKNOWN_FIELD.to_owned()))
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
}

// TODO: cache schedules
async fn fetch_class_info(
    course: Course,
    semester: Semester,
    section: &str,
) -> CommandResult<Option<ClassModel>> {
    let mut schedule_iter = ubs_lib::schedule_iter(course, semester).await?;

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
