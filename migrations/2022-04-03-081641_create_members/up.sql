-- Represent a Discord User
CREATE TABLE members
(
    id         INT PRIMARY KEY AUTO_INCREMENT,

    -- Basic info
    discord_id BIGINT UNSIGNED NOT NULL,
    guild_id   BIGINT UNSIGNED NOT NULL,
    username   VARCHAR(255)    NOT NULL,

    -- Xp metrics
    message_xp INT             NOT NULL DEFAULT 0,
    vocal_xp   INTEGER         NOT NULL DEFAULT 0,

    FOREIGN KEY (guild_id) REFERENCES guilds (id)
        ON DELETE CASCADE
);