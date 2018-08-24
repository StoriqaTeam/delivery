CREATE TABLE international_shipping (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL UNIQUE,
    companies JSONB NOT NULL DEFAULT '[]'
);
