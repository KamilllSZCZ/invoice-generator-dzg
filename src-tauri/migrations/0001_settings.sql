CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value_json TEXT NOT NULL
);

CREATE TABLE numbering_series (
    document_type TEXT PRIMARY KEY NOT NULL,
    prefix TEXT NOT NULL,
    format TEXT NOT NULL,
    counter_period TEXT,
    next_counter INTEGER NOT NULL CHECK (next_counter >= 0)
);

CREATE TABLE issued_numbers (
    id INTEGER PRIMARY KEY,
    document_type TEXT NOT NULL,
    formatted_number TEXT NOT NULL,
    issued_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(document_type, formatted_number)
);
