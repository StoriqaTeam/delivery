-- Your SQL goes here
CREATE TABLE countries (
    label VARCHAR PRIMARY KEY,
    name JSONB NOT NULL,
    parent_label VARCHAR,
    level INTEGER NOT NULL DEFAULT '0'
);
