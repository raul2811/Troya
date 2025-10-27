-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,

    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,

    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    document_number VARCHAR(50) NOT NULL,
    phone VARCHAR(20) NULL, -- 'Option<String>' se traduce a NULL (anulable)

    date_of_birth DATE NOT NULL, -- 'NaiveDate' se traduce a DATE
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    tb_account_id UUID NULL, -- Almacena el ID de la cuenta en TigerBeetle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);