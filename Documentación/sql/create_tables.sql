
-- DROP TABLE Projects; DROP Table Users; DROP TABLE AuthProviders; DROP TABLE Image; 

CREATE OR REPLACE TABLE AuthProviders (
    provider_id     INT         auto_increment UNIQUE,
    provider_name   VARCHAR(25) NOT NULL UNIQUE,

    -- Constraints
    PRIMARY KEY (provider_id)
);

CREATE OR REPLACE TABLE Image (
    image_id            INT     auto_increment UNIQUE,
    data                LONGBLOB NOT NULL,

    -- Constraints
    PRIMARY KEY (image_id)
);

CREATE OR REPLACE TABLE Users (
    user_id             INT         NOT NULL auto_increment,
    oauth_user_id       VARCHAR(50) NOT NULL UNIQUE,
    oauth_provider_id   INT         NOT NULL,
    

    -- Constraints
    PRIMARY KEY (user_id),
    CONSTRAINT fk_oauth_provider
        FOREIGN KEY (OAuth_provider_id) REFERENCES AuthProviders(provider_id)
);


CREATE TABLE IF NOT EXISTS Projects (
    project_id          INT     auto_increment UNIQUE,
    creator_user_id     INT     NOT NULL,
    image_id            INT     NOT NULL,
    project_data        JSON    NOT NULL,

    -- Constraints
    PRIMARY KEY (project_id),
    CONSTRAINT fk_creator_user_id
        FOREIGN KEY (creator_user_id) REFERENCES Users(user_id),
    CONSTRAINT fk_image_id
        FOREIGN KEY (image_id) REFERENCES Image(image_id)
);

