CREATE TABLE delivery_from (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    restriction_name VARCHAR NOT NULL
);

CREATE UNIQUE INDEX delivery_from_company_id_country_idx ON delivery_from (company_id, country);