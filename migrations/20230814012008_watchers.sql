CREATE TABLE watchers (
    user_id BIGINT NOT NULL,
    course TEXT NOT NULL,
    semester TEXT NOT NULL,
    career TEXT NOT NULL,
    section TEXT NOT NULL,
    UNIQUE (user_id, course, semester, career, section)
)
