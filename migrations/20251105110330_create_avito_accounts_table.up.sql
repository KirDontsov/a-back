-- Create avito_accounts table
CREATE TABLE IF NOT EXISTS avito_accounts (
    account_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id VARCHAR(255) NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    avito_client_secret TEXT NOT NULL,
    avito_client_id TEXT NOT NULL,
    is_connected BOOLEAN DEFAULT FALSE,
    created_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index for better query performance
CREATE INDEX IF NOT EXISTS idx_avito_accounts_user_id ON avito_accounts(user_id);