CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    discord_id VARCHAR(19) NOT NULL,
    mail VARCHAR NOT NULL,
    refresh_token TEXT NOT NULL
);