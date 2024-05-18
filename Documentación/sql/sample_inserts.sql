INSERT INTO AuthProviders(provider_name) values 
("Google"), ("GitHub");


SELECT provider_ID FROM AuthProviders where provider_name = "Google";

INSERT INTO Users(OAuth_provider_id, OAuth_user_id) values (1, "115262232359712602609");
INSERT INTO Users(OAuth_provider_id, OAuth_user_id) values (2, "215262232359712602609");


SELECT * FROM Users;