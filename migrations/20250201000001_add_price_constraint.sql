ALTER TABLE prices 
ADD CONSTRAINT unique_price_per_day 
UNIQUE (product_id, store_id, date, price);