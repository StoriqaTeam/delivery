CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL,
    store_id INTEGER NOT NULL,
    company_package_id INTEGER NOT NULL REFERENCES companies_packages (id) ON DELETE CASCADE,
    price DOUBLE PRECISION,
    deliveries_to JSONB NOT NULL DEFAULT '[]'
);
