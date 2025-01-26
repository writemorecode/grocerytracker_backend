ALTER TABLE stores ADD CONSTRAINT unique_store_location UNIQUE (street_number, street_name, city, country_code);
