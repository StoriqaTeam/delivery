CREATE TABLE company_delivery_from (
    id SERIAL PRIMARY KEY,
    company_id VARCHAR NOT NULL,
    country VARCHAR NOT NULL,
    company_restriction VARCHAR NOT NULL
);