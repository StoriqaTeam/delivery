CREATE TABLE restrictions (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    max_weight DOUBLE PRECISION DEFAULT '0' NOT NULL,
    max_size DOUBLE PRECISION DEFAULT '0' NOT NULL
);

CREATE UNIQUE INDEX restrictions_name_idx ON restrictions (name);

CREATE TABLE delivery_from (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    restriction_name VARCHAR NOT NULL
);

CREATE UNIQUE INDEX delivery_from_company_id_country_idx ON delivery_from (company_id, country);

CREATE TABLE delivery_to (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    additional_info JSONB
);

CREATE UNIQUE INDEX delivery_to_company_id_country_idx ON delivery_to (company_id, country);

CREATE TABLE local_shipping (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL UNIQUE,
    pickup BOOLEAN NOT NULL DEFAULT 'f',
    country VARCHAR NOT NULL,
    companies JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE international_shipping (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL UNIQUE,
    companies JSONB NOT NULL DEFAULT '[]'
);
