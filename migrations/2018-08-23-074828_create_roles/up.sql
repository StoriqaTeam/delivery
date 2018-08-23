DROP TABLE IF EXISTS user_roles;

CREATE TABLE roles (
    id      UUID    PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id INTEGER NOT NULL,
    name    VARCHAR NOT NULL,
    data    JSONB,

    CONSTRAINT role UNIQUE (user_id, name, data)
);

INSERT INTO roles (user_id, name) VALUES (1, 'superuser');
