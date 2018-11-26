CREATE TABLE shipping_rates (
    id SERIAL PRIMARY KEY,
    company_package_id INTEGER NOT NULL REFERENCES companies_packages (id) ON DELETE CASCADE,
    from_alpha3 VARCHAR NOT NULL,
    to_alpha3 VARCHAR NOT NULL,
    rates JSONB NOT NULL DEFAULT '[]'
);

CREATE UNIQUE INDEX shipping_rates_idx ON shipping_rates (company_package_id, from_alpha3, to_alpha3);

ALTER TABLE companies_packages DROP COLUMN shipping_rates;
