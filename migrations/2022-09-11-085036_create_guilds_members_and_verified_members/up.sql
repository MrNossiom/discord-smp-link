-- Represent a Discord guild.
CREATE TABLE `guilds`
(
    `id`                         BIGINT UNSIGNED NOT NULL,
    -- A guild name is limited to 100 characters.
    -- See https://discord.com/developers/docs/resources/user#usernames-and-nicknames
    `name`                       VARCHAR(100)    NOT NULL,
    `owner_id`                   BIGINT UNSIGNED NOT NULL,

    `verification_email_domain`  VARCHAR(100)    NULL,
    `verified_role_id`           BIGINT UNSIGNED NULL,

    `login_message_id`           BIGINT UNSIGNED NULL,
    `groups_message_id`          BIGINT UNSIGNED NULL,

    PRIMARY KEY (`id`)
);

-- Represent a class group of a verified member.
CREATE TABLE `levels`
(
    `id`       INTEGER         NOT NULL AUTO_INCREMENT,
    `name`     TEXT            NOT NULL,

    `guild_id` BIGINT UNSIGNED NOT NULL,
    `role_id`  BIGINT UNSIGNED NOT NULL,

    PRIMARY KEY (`id`),

    -- Guarantee that there is no name duplicates in the same guild.
    UNIQUE (`name`(255), `guild_id`),
    -- Guarantee that a role isn't used for multiple levels.
    UNIQUE (`guild_id`, `role_id`),

    FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);

-- Represent a class group of a verified member.
CREATE TABLE `classes`
(
    `id`       INTEGER         NOT NULL AUTO_INCREMENT,
    `name`     TEXT            NOT NULL,
    `level_id` INTEGER         NOT NULL,

    `guild_id` BIGINT UNSIGNED NOT NULL,
    `role_id`  BIGINT UNSIGNED NOT NULL,

    PRIMARY KEY (`id`),

    -- Guarantee that there is no name duplicates in the same guild.
    UNIQUE (`name`(255), `guild_id`),
    -- Guarantee that a role isn't used for multiple classes.
    UNIQUE (`guild_id`, `role_id`),

    FOREIGN KEY (`level_id`) REFERENCES `levels` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE,

    FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);

-- Represent a guild member.
CREATE TABLE `members`
(
    `id`         INTEGER         NOT NULL AUTO_INCREMENT,
    `guild_id`   BIGINT UNSIGNED NOT NULL,
    `discord_id` BIGINT UNSIGNED NOT NULL,
    -- A Discord username is limited to 32 characters.
    `username`   VARCHAR(32)     NOT NULL,

    -- Metrics
    `message_xp` INTEGER         NOT NULL DEFAULT 0,
    `vocal_xp`   INTEGER         NOT NULL DEFAULT 0,

    PRIMARY KEY (`id`),

    UNIQUE (`guild_id`, `discord_id`),

    FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);

-- Represent a guild member that verified his identity.
CREATE TABLE `verified_members`
(
    `member_id`  INTEGER      NOT NULL,

    -- An E-Mail is limited to 254 characters.
    -- See RFC Errata 1690: https://www.rfc-editor.org/errata/eid1690
    `mail`       VARCHAR(256) NOT NULL,
    `first_name` TEXT         NOT NULL,
    `last_name`  TEXT         NOT NULL,

    `class_id`   INTEGER      NOT NULL,

    PRIMARY KEY (`member_id`),

    FOREIGN KEY (`member_id`) REFERENCES `members` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    FOREIGN KEY (`class_id`) REFERENCES `classes` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);