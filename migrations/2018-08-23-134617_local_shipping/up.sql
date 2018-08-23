CREATE TABLE local_shipping (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL UNIQUE,
    pickup BOOLEAN NOT NULL DEFAULT 'f',
    country VARCHAR NOT NULL,
    companies JSONB NOT NULL DEFAULT '[]'
);
