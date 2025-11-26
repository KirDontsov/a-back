-- Create avito_feeds table
CREATE TABLE IF NOT EXISTS avito_feeds (
    feed_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    account_id UUID NOT NULL,
    category VARCHAR(255) NOT NULL,
    created_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create avito_ads table
CREATE TABLE IF NOT EXISTS avito_ads (
    ad_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    feed_id UUID NOT NULL REFERENCES avito_feeds(feed_id) ON DELETE CASCADE,
    avito_ad_id VARCHAR(255),
    parsed_id VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create avito_ad_fields table
CREATE TABLE IF NOT EXISTS avito_ad_fields (
    field_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    ad_id UUID NOT NULL REFERENCES avito_ads(ad_id) ON DELETE CASCADE,
    tag VARCHAR(255) NOT NULL,
    data_type VARCHAR(50) NOT NULL DEFAULT 'string',
    field_type VARCHAR(50) NOT NULL DEFAULT 'attribute',
    created_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create avito_ad_field_values table
CREATE TABLE IF NOT EXISTS avito_ad_field_values (
    field_value_id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    field_id UUID NOT NULL REFERENCES avito_ad_fields(field_id) ON DELETE CASCADE,
    value TEXT,
    created_ts TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_avito_ads_feed_id ON avito_ads(feed_id);
CREATE INDEX IF NOT EXISTS idx_avito_ads_parsed_id ON avito_ads(parsed_id);
CREATE INDEX IF NOT EXISTS idx_avito_ads_avito_ad_id ON avito_ads(avito_ad_id);
CREATE INDEX IF NOT EXISTS idx_avito_ad_fields_ad_id ON avito_ad_fields(ad_id);
CREATE INDEX IF NOT EXISTS idx_avito_ad_fields_tag ON avito_ad_fields(tag);
CREATE INDEX IF NOT EXISTS idx_avito_ad_field_values_field_id ON avito_ad_field_values(field_id);