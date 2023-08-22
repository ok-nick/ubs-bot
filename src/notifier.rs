use poise::serenity_prelude::{ChannelId, CreateMessage, Http, UserId};

use crate::{
    cache::{ClassRecord, Query},
    Context,
};

const TIME_FORMAT: &str = "%-I:%M%p";
const UNKNOWN_FIELD: &str = "[unknown]";

#[derive(Debug)]
pub struct Notifier {
    new: ClassRecord,
    user_ids: Vec<UserId>,
    old: Option<ClassRecord>,
    query: Query,
}

impl Notifier {
    pub(crate) fn new(
        new: ClassRecord,
        user_ids: Vec<UserId>,
        query: Query,
        old: Option<ClassRecord>,
    ) -> Notifier {
        Notifier {
            new,
            user_ids,
            query,
            old,
        }
    }
    pub fn old_record(&self) -> &Option<ClassRecord> {
        &self.old
    }

    pub fn new_record(&self) -> &ClassRecord {
        &self.new
    }

    pub async fn notify_reply(&self, ctx: Context<'_>) -> Result<(), crate::Error> {
        Ok(())
    }

    // TODO: add more context about the old/new changes
    pub async fn notify(&self, http: &Http, channel: ChannelId) -> Result<(), crate::Error> {
        if let Some(old) = &self.old {
            channel
                .send_message(http, |f| {
                    info_msg_users(f, &self.query, old, &self.user_ids)
                })
                .await?;
        }
        channel
            .send_message(http, |f| {
                info_msg_users(f, &self.query, &self.new, &self.user_ids)
            })
            .await?;

        Ok(())
    }
}

fn info_msg_users<'a, 'b>(
    f: &'a mut CreateMessage<'b>,
    query: &Query,
    record: &ClassRecord,
    user_ids: &[UserId],
) -> &'a mut CreateMessage<'b> {
    info_msg(f, query, record).allowed_mentions(|am| am.empty_parse().users(user_ids))
}

fn info_msg<'a, 'b>(
    f: &'a mut CreateMessage<'b>,
    query: &Query,
    record: &ClassRecord,
) -> &'a mut CreateMessage<'b> {
    let model = &record.model;
    f.embed(|e| {
        e.title(format!("{} - {}", query.course, query.semester))
            .author(|a| a.name(model.instructor.as_deref().unwrap_or(UNKNOWN_FIELD)))
            .timestamp(record.timestamp)
            // .field("Id", model.class_id.unwrap(), true)
            .field(
                "Section",
                model.section.as_deref().unwrap_or(UNKNOWN_FIELD),
                true,
            )
            .field(
                "Type",
                model
                    .class_type
                    .map(|x| x.to_string())
                    .as_deref()
                    .unwrap_or(UNKNOWN_FIELD),
                true,
            )
            .field("Room", model.room.as_deref().unwrap_or(UNKNOWN_FIELD), true)
            .field(
                "Open",
                model
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
                    model
                        .open_seats
                        .map(|x| x.to_string())
                        .as_deref()
                        .unwrap_or(UNKNOWN_FIELD),
                    model
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
                match model.days_of_week {
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
                    model
                        .start_time
                        .map(|x| x.format(TIME_FORMAT).to_string())
                        .as_deref()
                        .unwrap_or(UNKNOWN_FIELD),
                    model
                        .end_time
                        .map(|x| x.format(TIME_FORMAT).to_string())
                        .as_deref()
                        .unwrap_or(UNKNOWN_FIELD),
                ),
                true,
            )
    })
}
