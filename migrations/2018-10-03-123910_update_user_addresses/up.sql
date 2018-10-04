ALTER TABLE user_addresses
ADD COLUMN country_code VARCHAR NULL;

UPDATE user_addresses
SET country_code = countries.alpha3
FROM countries
WHERE user_addresses.country = countries.label;
