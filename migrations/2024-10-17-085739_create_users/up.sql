-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE users (
                       id TEXT PRIMARY KEY,
                       name VARCHAR NOT NULL,
                       email TEXT NOT NULL UNIQUE,
                       password TEXT,
                       verified BOOLEAN NOT NULL DEFAULT FALSE,
                       created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMP
);

CREATE INDEX users_email_idx ON users(email);

SELECT diesel_manage_updated_at('users');