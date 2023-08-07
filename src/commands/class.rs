use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    futures::TryStreamExt,
    model::prelude::*,
    prelude::*,
};
use ubs_lib::parser::{Class, ClassSchedule};

#[command]
#[sub_commands(info, watch, unwatch)]
pub async fn class(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx.http, "Please specify a subcommand.").await?;

    Ok(())
}

// TODO:
//    [prefix]class info [course] [semester] [section]
//    [prefix]class info cse115 spring2023 a5
#[command]
#[description("Get information for class.")]
pub async fn info(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.start_typing(&ctx.http)?;

    let course = args.single::<String>()?;
    let semester = args.single::<String>()?;
    let section = args.single::<String>()?;

    match fetch_class_info(&course, &semester, &section).await? {
        Some(handle) => {
            let class = handle.get();

            // TODO: ubs-lib should probably return Options instead of Results..
            //       How is a result useful when we know it was a faulty match?
            let instructor = class.instructor()?;
            let open = class.is_open()?;
            let class_type = class.class_type()?;
            let class_id = class.class_id()?;
            let section = class.section()?;
            let days_of_week = class.days_of_week()?;
            let start_time = class.start_time()?;
            let end_time = class.end_time()?;
            let room = class.room()?;
            let open_seats = class.open_seats()?;
            let total_seats = class.total_seats()?;

            // TODO: set timestamp to nearest class start date?
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(course)
                            .author(|a| a.name(instructor))
                            .field("Id", class_id, false)
                            .field("Section", section, false)
                            .field("Type", class_type, false)
                            .field("Open", open, false)
                            .field("Room", room, false)
                        // TODO: fix these signatures in the ubs-lib crate
                        // .field("Days of Week", days_of_week, false)
                        // .field("Start Time", start_time, true)
                        // .field("End Time", end_time, true)
                        // .field("Open Seats", open_seats, true)
                        // .field("Total Seats", total_seats, true)
                    })
                })
                .await?;
        }
        None => {
            msg.channel_id
                .say(&ctx.http, "Could not find the requested class.")
                .await?;
        }
    }

    Ok(())
}

// TODO:
//    [prefix]class watch [course] [semester] [section]
//    [prefix]class watch cse115 spring2023 a5
#[command]
#[description("Notify when class opens.")]
pub async fn watch(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.channel_id.start_typing(&ctx.http)?;
    todo!()
}

// TODO:
//    [prefix]class unwatch [course] [semester] [section]
//    [prefix]class unwatch cse115 spring2023 a5
#[command]
#[description("Stop notifying when class opens.")]
pub async fn unwatch(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    msg.channel_id.start_typing(&ctx.http)?;
    todo!()
}

struct ClassHandle {
    schedule: ClassSchedule,
    group_num: u32,
    class_num: u32,
}

impl ClassHandle {
    pub fn get(&self) -> Class<'_> {
        let group = self.schedule.group_from_index(self.group_num);
        group.class_from_index(self.class_num)
    }
}

// TODO: cache schedules
async fn fetch_class_info(
    course: &str,
    semester: &str,
    section: &str,
) -> CommandResult<Option<ClassHandle>> {
    let mut schedule_iter = ubs_lib::schedule_iter(course.parse()?, semester.parse()?).await?;

    #[allow(clippy::never_loop)] // TODO: temp
    while let Some(schedule) = schedule_iter.try_next().await? {
        let schedule = schedule?;
        if let Some((group_num, class_num)) = class_from_schedule(section, &schedule)? {
            return Ok(Some(ClassHandle {
                schedule,
                group_num,
                class_num,
            }));
        }

        break;
    }

    Ok(None)
}

fn class_from_schedule(
    section: &str,
    schedule: &ClassSchedule,
) -> CommandResult<Option<(u32, u32)>> {
    for (group_num, group) in schedule.group_iter().enumerate() {
        for (class_num, class) in group.class_iter().enumerate() {
            if class.section()? == section {
                return Ok(Some((group_num as u32, class_num as u32)));
            }
        }
    }
    Ok(None)
}
