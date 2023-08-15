use serenity::futures::TryStreamExt;
// TODO: struct that manages caching and propagating changes to watchers
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Json,
    },
    PgPool,
};
use ubs_lib::{model::ClassModel, parser::ClassSchedule, Career, Course, Semester};

// TODO: struct to contain user queries
#[derive(Debug, Clone)]
pub struct Query {
    course: String,
    semester: String,
    career: String,
    section: String,
}

#[derive(Debug)]
pub struct ClassRecord {
    pub timestamp: DateTime<Utc>,
    pub model: ClassModel,
}
#[derive(Debug)]
pub struct Cache {
    pool: PgPool,
}

impl Query {
    pub fn new(course: Course, semester: Semester, career: Career, section: String) -> Query {
        Query {
            // TODO: remove clones
            course: course.id().to_owned(),
            semester: semester.id().to_owned(),
            career: career.id().to_owned(),
            section: section.to_uppercase(),
        }
    }

    pub fn from_raw(
        course_id: String,
        semester_id: String,
        career_id: String,
        section: String,
    ) -> Query {
        Query {
            course: course_id,
            semester: semester_id,
            career: career_id,
            section: section.to_uppercase(),
        }
    }
}

impl Cache {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, query: Query) -> Result<ClassRecord, FetchClassError> {
        let latest_rec = sqlx::query!(
            r#"
SELECT timestamp, data as "data: Json<ClassModel>"
FROM cache
WHERE
  $1 in (course)
  AND
  $2 in (semester)
  AND
  $3 in (career)
  AND
  $4 in (section)
ORDER BY timestamp DESC;
            "#,
            query.course,
            query.semester,
            query.career,
            query.section
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ClassRecord {
            timestamp: latest_rec.timestamp,
            model: latest_rec.data.0,
        })
    }

    pub async fn update(&self, query: Query) -> Result<ClassModel, FetchClassError> {
        let info = self.fetch(query.clone()).await?;
        sqlx::query!(
            "INSERT INTO cache VALUES ($1, $2, $3, $4, $5, $6);",
            Utc::now(),
            query.course,
            query.semester,
            query.career,
            query.section,
            Json(info.clone()) as _
        )
        .execute(&self.pool)
        .await?;
        Ok(info)
    }

    pub async fn fetch(&self, query: Query) -> Result<ClassModel, FetchClassError> {
        let mut schedule_iter = ubs_lib::schedule_iter_with_career(
            Course::Raw(query.course),
            Semester::Raw(query.semester),
            Career::Raw(query.career),
        )
        .await?;

        #[allow(clippy::never_loop)] // TODO: temp
        while let Some(schedule) = schedule_iter.try_next().await? {
            let schedule = schedule?;
            let class = self.class_from_schedule(&query.section, &schedule);
            match class {
                Ok(class) => return Ok(class),
                Err(e) => match e {
                    FetchClassError::SectionNotFound(_) => break,
                    _ => {}
                },
            }

            break;
        }

        Err(FetchClassError::SectionNotFound(query.section))
    }

    fn class_from_schedule(
        &self,
        section: &str,
        schedule: &ClassSchedule,
    ) -> Result<ClassModel, FetchClassError> {
        for group in schedule.group_iter() {
            for class in group.class_iter() {
                if class.section()? == section {
                    return Ok(class.model()?);
                }
            }
        }

        Err(FetchClassError::SectionNotFound(section.to_owned()))
    }
}

// TODO: fix up
#[derive(Debug, thiserror::Error)]
pub enum FetchClassError {
    #[error(transparent)]
    Schedule(#[from] ubs_lib::ScheduleError),
    #[error(transparent)]
    Parse(#[from] ubs_lib::parser::ParseError),
    #[error(transparent)]
    Session(#[from] ubs_lib::session::SessionError),
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error("session {0} was not found")]
    SectionNotFound(String),
}
