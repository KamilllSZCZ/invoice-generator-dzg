CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT CHECK (id > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 200),
    tax_id TEXT,
    address TEXT,
    postal_code TEXT,
    city TEXT,
    country TEXT,
    email TEXT,
    phone TEXT,
    notes TEXT,
    search_text TEXT NOT NULL
);
