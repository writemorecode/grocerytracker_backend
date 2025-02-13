ALTER TABLE stores ADD COLUMN address TEXT;

UPDATE stores SET address = CONCAT_WS(' ',  street_name, street_number);

ALTER TABLE stores ALTER COLUMN address SET NOT NULL;

ALTER TABLE stores DROP CONSTRAINT unique_store_location;

ALTER TABLE stores ADD CONSTRAINT unique_store_location UNIQUE (address, city, country_code);

ALTER TABLE stores DROP COLUMN street_number;

ALTER TABLE stores DROP COLUMN street_name;
