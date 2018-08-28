CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    label VARCHAR NOT NULL,
    description VARCHAR,
    deliveries_from JSONB NOT NULL DEFAULT '[]',
    logo VARCHAR NOT NULL
);
