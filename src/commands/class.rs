use poise::serenity_prelude::futures::TryStreamExt;
use poise::CreateReply;
use ubs_lib::{model::ClassModel, parser::ClassSchedule, Course, Semester};

use crate::Context;

const TIME_FORMAT: &str = "%-I:%M%p";
const UNKNOWN_FIELD: &str = "[unknown]";

// #[description("Get information of class")]
#[poise::command(slash_command)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "test"] course: String,
    #[description = "test"] semester: String,
    #[description = "test"] section: String,
    #[description = "test"] career: Option<String>,
) -> Result<(), crate::Error> {
    ctx.defer().await?;

    let section = section.to_uppercase();

    match fetch_class_info(course.parse()?, semester.parse()?, &section).await? {
        Some(class) => {
            ctx.send(|f| info_msg(f, &class, &course, &semester))
                .await?;
        }
        None => {
            // TODO: use embeds
            ctx.say(format!("Could not find {course}, section {section}, during {semester}.

*Does this class exist? It may just be missing a mapping.
Read here for more information: https://github.com/ok-nick/ubs-bot#why-cant-it-find-a-class-that-i-know-exists*"),
                )
                .await?;
        }
    }

    Ok(())
}

// TODO: broadcast_typing macro
// TODO: insane boilerplate
// #[aliases("raw")] // TODO: can I make it so raw is the only way to call it?
// #[description("Get information of class using raw ids")]
#[poise::command(slash_command)]
pub async fn rawinfo(
    ctx: Context<'_>,
    #[description = "test"] course: String,
    #[description = "test"] semester: String,
    #[description = "test"] section: String,
    #[description = "test"] career: Option<String>,
) -> Result<(), crate::Error> {
    ctx.defer().await?;

    let section = section.to_uppercase();

    // TODO: in `ubs-lib` impl Display on ids to avoid clone
    // TODO: use cache
    match fetch_class_info(
        Course::Raw(course.clone()),
        Semester::Raw(semester.clone()),
        &section,
    )
    .await?
    {
        Some(class) => {
            ctx.send(|f| info_msg(f, &class, &course, &semester))
                .await?;
        }
        None => {
            ctx.say(
                    format!("Could not find course id {course}, section {section}, during semester id {semester}."),
                )
                .await?;
        }
    }

    Ok(())
}

// #[description("Notify when class opens")]
#[poise::command(slash_command)]
pub async fn watch(
    ctx: Context<'_>,
    #[description = "test"] course: String,
    #[description = "test"] semester: String,
    #[description = "test"] section: String,
    #[description = "test"] career: Option<String>,
) -> Result<(), crate::Error> {
    ctx.defer().await?;

    let section = section.to_uppercase();

    sqlx::query!(
        "INSERT INTO watchers VALUES ($1, $2, $3, $4, $5)",
        ctx.author().id.0 as i64,
        course,
        semester,
        career,
        section
    )
    .execute(ctx.data().cache.database())
    .await?;

    Ok(())
}

// TODO: maybe this command should take an integer and there should be another command that lists watches
// #[description("Stop notifying when class opens")]
#[poise::command(slash_command)]
pub async fn unwatch(
    ctx: Context<'_>,
    #[description = "test"] course: String,
    #[description = "test"] semester: String,
    #[description = "test"] section: String,
    #[description = "test"] career: Option<String>,
) -> Result<(), crate::Error> {
    ctx.defer().await?;

    let section = section.to_uppercase();

    todo!()
}

// TODO: absolute monstrosity of a function
fn info_msg<'a, 'b>(
    f: &'a mut CreateReply<'b>,
    class: &ClassModel,
    course: &str,
    semester: &str,
) -> &'a mut CreateReply<'b> {
    f.embed(|e| {
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
//       anytime a class is fetched the watchers should check for updates
async fn fetch_class_info(
    course: Course,
    semester: Semester,
    section: &str,
) -> Result<Option<ClassModel>, crate::Error> {
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
) -> Result<Option<ClassModel>, crate::Error> {
    for group in schedule.group_iter() {
        for class in group.class_iter() {
            if class.section()? == section {
                return Ok(Some(class.model()?));
            }
        }
    }

    Ok(None)
}
