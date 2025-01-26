CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE stores (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    street_number INT NOT NULL,
    street_name TEXT NOT NULL,
    city TEXT NOT NULL,
    country_code TEXT NOT NULL,
    coordinate GEOMETRY(Point, 4326) NOT NULL
);

ALTER TABLE stores ADD CONSTRAINT unique_store_location UNIQUE (street_number, street_name, city, country_code);
