-- Add migration script here
CREATE TABLE IF NOT EXISTS transaction_history (
    id UUID PRIMARY KEY,

    -- Esta es la "relación" clave con nuestra tabla de usuarios
    -- ON DELETE CASCADE significa que si un usuario es borrado, su historial también
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Estos IDs viven en TigerBeetle, así que no hay REFERENCES
    from_account_id UUID NOT NULL,
    to_account_id UUID NOT NULL,

    -- Usamos BIGINT para i64 (el 'amount_cents')
    amount_cents BIGINT NOT NULL,

    description VARCHAR(255) NULL, -- 'Option<String>'
    status VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);