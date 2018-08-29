CREATE TABLE companies_packages (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    package_id INTEGER NOT NULL REFERENCES packages (id) ON DELETE CASCADE
);
