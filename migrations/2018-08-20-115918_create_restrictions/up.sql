CREATE TABLE restrictions (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    max_weight DOUBLE PRECISION DEFAULT '0',
    max_size DOUBLE PRECISION DEFAULT '0'
);

CREATE UNIQUE INDEX restrictions_name_idx ON restrictions (name);