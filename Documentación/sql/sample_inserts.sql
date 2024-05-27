INSERT INTO AuthProviders(provider_name) values 
("Google"), ("GitHub");


SELECT provider_ID FROM AuthProviders where provider_name = "Google";

-- TRUNCATE TABLE Users; DELETE FROM Users;
INSERT INTO Users(OAuth_provider_id, OAuth_user_id) values (1, "115262232359712602609");
INSERT INTO Users(OAuth_provider_id, OAuth_user_id) values (2, "215262232359712602609");


SELECT * FROM Users;

SELECT * FROM Projects;

-- INSERT INTO Image(data) values (0x00)
SELECT * From Image