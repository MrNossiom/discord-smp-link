-- Represent a user that has been verified via Google Authentication
CREATE TABLE verified_members
(
    -- Primary and foreign key
    id         INT PRIMARY KEY AUTO_INCREMENT,
    user_id    INT          NOT NULL,

    -- User details
    first_name VARCHAR(255) NOT NULL,
    last_name  VARCHAR(255) NOT NULL,
    mail       VARCHAR(255) NOT NULL,

    FOREIGN KEY (user_id) REFERENCES members (id)
        ON DELETE CASCADE
);