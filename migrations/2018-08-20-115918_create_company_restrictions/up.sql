CREATE TABLE company_restrictions (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    max_weight DOUBLE PRECISION DEFAULT '0',
    max_size DOUBLE PRECISION DEFAULT '0'
);

CREATE UNIQUE INDEX company_restrictions_name_idx ON company_restrictions (name);