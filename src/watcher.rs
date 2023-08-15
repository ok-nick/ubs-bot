use std::time::Duration;

use sqlx::{types::chrono::Utc, PgPool};

use crate::cache::{Cache, Query};

#[derive(Debug)]
pub struct Watcher {
    pool: PgPool,
    cache: Cache,
    update_interval: Duration,
    check_interval: Duration,
}

impl Watcher {
    pub fn new(pool: PgPool, update_interval: Duration, check_interval: Duration) -> Watcher {
        Watcher {
            pool: pool.clone(),
            cache: Cache::new(pool),
            update_interval,
            check_interval,
        }
    }

    pub fn notify(&self, user_id: u64) {
        // TODO: notify users of changes
    }

    pub async fn watch(&self) {
        loop {
            let queries = sqlx::query!(
                r#"
SELECT course, semester, career, section
FROM watchers
GROUP BY (course, semester, career, section);
                "#
            )
            .fetch_all(&self.pool)
            .await
            .unwrap(); // TODO: handle

            // TODO: check query for updates (need cache)
            //       if update then notify users

            for query_rec in queries {
                // TODO; query take reference to values?
                // TODO: create function that doesn't json decode value and just returns timestamp
                let last_query_rec = self
                    .cache
                    .get(Query::from_raw(
                        query_rec.course.clone(),
                        query_rec.semester.clone(),
                        query_rec.career.clone(),
                        query_rec.section.clone(),
                    ))
                    .await
                    .unwrap(); // TODO

                if Utc::now()
                    .signed_duration_since(last_query_rec.timestamp)
                    .to_std()
                    .unwrap() // TODO
                    > self.update_interval
                {
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
                    .fetch_all(&self.pool)
                    .await
                    .unwrap(); // TODO: handle

                    for user_id_rec in user_ids {
                        self.notify(user_id_rec.user_id as u64)
                    }
                }
            }

            tokio::time::sleep(self.check_interval).await;
        }
    }
}
