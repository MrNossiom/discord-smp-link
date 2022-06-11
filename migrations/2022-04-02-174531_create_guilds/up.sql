CREATE TABLE guilds
(
    id               BIGINT UNSIGNED PRIMARY KEY,

    name             VARCHAR(255)    NOT NULL,
    owner_id         BIGINT UNSIGNED NOT NULL,

    setup_message_id BIGINT UNSIGNED
);
