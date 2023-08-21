use std::time::Duration;

use sqlx::{types::chrono::Utc, PgPool};
use ubs_lib::model::ClassModel;

use crate::cache::{Cache, Query};

#[derive(Debug)]
pub struct Watcher {
    database: PgPool,
    cache: Cache,
    update_interval: Duration,
}

impl Watcher {
    pub fn new(database: PgPool, cache: Cache, update_interval: Duration) -> Watcher {
        Watcher {
            database: database.clone(),
            cache,
            update_interval,
        }
    }

    pub async fn notify(&self, model: ClassModel, user_ids: &[u64]) {
        // TODO: ping all users with new class using `info_msg` (move func to here)
    }

    pub async fn watch(&self) {
        loop {
            self.check().await;
        }
    }

    // TODO: would be nicer if I returned a iterator/vec of users/courses
    pub async fn check(&self) {
        let queries = sqlx::query!(
            r#"
SELECT course, semester, career, section
FROM watchers
GROUP BY (course, semester, career, section);
                "#
        )
        .fetch_all(&self.database)
        .await
        .unwrap(); // TODO: handle

        for query_rec in queries {
            // TODO; query take reference to values?
            let model = self
                .cache
                .get_or_update(&Query::from_raw(
                    query_rec.course.clone(),
                    query_rec.semester.clone(),
                    query_rec.career.clone(),
                    query_rec.section.clone(),
                ))
                .await
                .unwrap(); // TODO
                           // TODO: having to pass 100 ids like this is ugly, can I store 1 id to match all the ids?
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
            .fetch_all(&self.database)
            .await
            .unwrap(); // TODO: handle

            self.notify(
                model,
                &user_ids
                    .iter()
                    .map(|x| x.user_id as u64)
                    .collect::<Vec<u64>>(),
            )
            .await;
        }

        tokio::time::sleep(self.update_interval).await;
    }
}
