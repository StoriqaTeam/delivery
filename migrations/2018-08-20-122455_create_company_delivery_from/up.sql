CREATE TABLE company_delivery_from (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    company_restriction VARCHAR NOT NULL
);

CREATE UNIQUE INDEX company_delivery_from_company_id_country_idx ON company_delivery_from (company_id, country);