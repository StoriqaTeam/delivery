DROP TABLE IF EXISTS shipping_rates;

ALTER TABLE companies_packages ADD COLUMN shipping_rates jsonb;
