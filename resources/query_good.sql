SELECT
    *,
    DATE_PART('year', AGE(date_of_birth)) AS age
FROM customer
WHERE date_of_birth = CURRENT_DATE
LIMIT $limit::INT;