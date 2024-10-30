-- Your SQL goes here
CREATE TABLE posts (
                       id TEXT PRIMARY KEY,
                       title VARCHAR NOT NULL UNIQUE,
                       body TEXT NOT NULL,
                       published BOOLEAN NOT NULL DEFAULT FALSE,
                       user_id TEXT REFERENCES users(id),
                       created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMP
);

CREATE INDEX posts_title_idx ON posts(title);

SELECT diesel_manage_updated_at('posts');