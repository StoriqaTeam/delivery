CREATE TABLE company_delivery_to (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    additional_info JSONB
);

CREATE UNIQUE INDEX company_delivery_to_company_id_country_idx ON company_delivery_to (company_id, country);