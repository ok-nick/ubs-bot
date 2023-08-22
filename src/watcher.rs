use std::time::Duration;

use poise::serenity_prelude::UserId;

use crate::{
    cache::{Cache, ClassRecord, ClassUpdate, Query},
    notifier::Notifier,
};

#[derive(Debug)]
pub enum Check {
    Old(ClassRecord),
    New(Box<Notifier>), // might as well box it up to reduce footprint
}

#[derive(Debug)]
pub struct Watcher {
    cache: Cache,
}

impl Watcher {
    pub fn new(cache: Cache) -> Watcher {
        Watcher { cache }
    }

    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    pub async fn watch(&self, interval: Duration, max_age: Duration) {
        loop {
            // TODO: notify checks
            self.check_all(max_age).await;
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn check_all(&self, max_age: Duration) -> Vec<Check> {
        let queries = sqlx::query!(
            r#"
SELECT course, semester, career, section
FROM watchers
GROUP BY (course, semester, career, section);
                "#
        )
        .fetch_all(self.cache.database())
        .await
        .unwrap(); // TODO: handle

        let mut checks = Vec::new();
        for rec in queries {
            checks.push(
                self.check(
                    Query::from_ids(rec.course, rec.semester, rec.career, rec.section),
                    max_age,
                )
                .await,
            );
        }

        checks
    }

    pub async fn check(&self, query: Query, max_age: Duration) -> Check {
        let update = self.cache.get_or_update(&query, max_age).await.unwrap();
        match update {
            ClassUpdate::Old(old) => Check::Old(old),
            ClassUpdate::New { old, new } => Check::New(Box::new(Notifier::new(
                new,
                self.watchers(&query).await,
                query,
                old,
            ))),
        }
    }

    pub async fn watchers(&self, query: &Query) -> Vec<UserId> {
        sqlx::query!(
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
            query.course,
            query.semester,
            query.career,
            query.section,
        )
        .fetch_all(self.cache.database())
        .await
        .unwrap() // TODO: handle
        .iter()
        .map(|x| UserId(x.user_id as u64))
        .collect()
    }
}
