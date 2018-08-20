CREATE TABLE delivery_to (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    additional_info JSONB
);

CREATE UNIQUE INDEX delivery_to_company_id_country_idx ON delivery_to (company_id, country);