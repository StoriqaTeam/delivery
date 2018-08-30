CREATE TABLE pickups (
    id SERIAL PRIMARY KEY,
    base_product_id INTEGER NOT NULL,
    store_id INTEGER NOT NULL,
    pickup BOOLEAN NOT NULL,
    price DOUBLE PRECISION
);
