CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    discord_id VARCHAR(19) NOT NULL,
    full_name VARCHAR NOT NULL,
    mail VARCHAR NOT NULL,
    refresh_token VARCHAR NOT NULL,
    access_token VARCHAR NOT NULL,
    expires_at TIMESTAMP NOT NULL
);