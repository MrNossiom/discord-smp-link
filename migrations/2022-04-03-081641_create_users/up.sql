-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    mail VARCHAR,
    refresh_token TEXT
);