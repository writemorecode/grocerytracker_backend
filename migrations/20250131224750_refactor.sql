CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    barcode TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS stores (
    id SERIAL PRIMARY KEY,
    name TEXT,
    street_number INT,
    street_name TEXT,
    city TEXT,
    country_code TEXT,
    coordinate GEOMETRY(Point, 4326) NOT NULL
);

ALTER TABLE stores ADD CONSTRAINT unique_store_location
UNIQUE (street_number, street_name, city, country_code);

CREATE TABLE IF NOT EXISTS prices (
    id SERIAL PRIMARY KEY,
    price REAL NOT NULL,
    date DATE DEFAULT CURRENT_DATE,
    product_id INTEGER REFERENCES products(id),
    store_id INTEGER REFERENCES stores(id)
);

