DROP TABLE watchers;
DROP TABLE cache;

CREATE TABLE watchers (
    user_id BIGINT NOT NULL,
    course TEXT NOT NULL,
    semester TEXT NOT NULL,
    career TEXT NOT NULL,
    section TEXT NOT NULL,
    UNIQUE (user_id, course, semester, career, section)
);

CREATE TABLE cache (
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    course TEXT NOT NULL,
    semester TEXT NOT NULL,
    career TEXT NOT NULL,
    section TEXT NOT NULL,
    data JSONB NOT NULL
)