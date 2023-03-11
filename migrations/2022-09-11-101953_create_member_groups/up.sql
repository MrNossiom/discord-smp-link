-- Represent a group related to a specific subject.
CREATE TABLE `groups`
(
    `id`       INTEGER         NOT NULL AUTO_INCREMENT,
    `name`     TEXT            NOT NULL,
    `emoji`    CHAR,

    `guild_id` BIGINT UNSIGNED NOT NULL,
    `role_id`  BIGINT UNSIGNED NOT NULL,

    PRIMARY KEY (`id`),

    -- Guarantee that there is no name duplicates in the same guild.
    UNIQUE (`name`(255), `guild_id`),
    -- Guarantee that a role isn't used for multiple groups.
    UNIQUE (`guild_id`, `role_id`),

    FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);

-- Represent Many-to-Many relationship between verified members and groups.
CREATE TABLE `groups_of_verified_members`
(
    `verified_member_id` INTEGER NOT NULL,
    `group_id`           INTEGER NOT NULL,

    PRIMARY KEY (`verified_member_id`, `group_id`),

    UNIQUE (`verified_member_id`, `group_id`),

    FOREIGN KEY (`verified_member_id`) REFERENCES `verified_members` (`member_id`)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    FOREIGN KEY (`group_id`) REFERENCES `groups` (`id`)
        ON DELETE RESTRICT ON UPDATE CASCADE
);
