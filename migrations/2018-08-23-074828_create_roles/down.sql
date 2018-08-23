DROP TABLE IF EXISTS roles;

CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    role VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX user_roles_user_id_idx ON user_roles (user_id);

SELECT diesel_manage_updated_at('user_roles');

INSERT INTO user_roles (user_id, role) VALUES (1, 'superuser') ON CONFLICT (user_id) DO NOTHING;
