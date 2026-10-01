-- Initial Schema for Astrea SDA API with Row-Level Security (RLS)

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'viewer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS satellites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    line_one TEXT NOT NULL,
    line_two TEXT NOT NULL,
    owner_id TEXT,
    created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tle_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    satellite_id UUID NOT NULL REFERENCES satellites(id) ON DELETE CASCADE,
    line_one TEXT NOT NULL,
    line_two TEXT NOT NULL,
    epoch TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_satellites_name ON satellites (LOWER(name));
CREATE INDEX IF NOT EXISTS idx_tle_history_sat_epoch ON tle_history (satellite_id, epoch DESC);

-- Enable Row-Level Security (RLS)
ALTER TABLE satellites ENABLE ROW LEVEL SECURITY;
ALTER TABLE tle_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- RLS Policies
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE policyname = 'Public or Authenticated Read Satellites') THEN
        CREATE POLICY "Public or Authenticated Read Satellites" ON satellites FOR SELECT USING (true);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE policyname = 'Editor and Admin Write Satellites') THEN
        CREATE POLICY "Editor and Admin Write Satellites" ON satellites FOR ALL USING (
            current_setting('request.jwt.claim.role', true) IN ('editor', 'admin', 'service_role')
            OR current_setting('request.jwt.claims', true)::json->>'role' IN ('editor', 'admin', 'service_role')
            OR current_setting('role', true) = 'postgres'
        );
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE policyname = 'Public or Authenticated Read TLE History') THEN
        CREATE POLICY "Public or Authenticated Read TLE History" ON tle_history FOR SELECT USING (true);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_policies WHERE policyname = 'Editor and Admin Write TLE History') THEN
        CREATE POLICY "Editor and Admin Write TLE History" ON tle_history FOR ALL USING (
            current_setting('request.jwt.claim.role', true) IN ('editor', 'admin', 'service_role')
            OR current_setting('request.jwt.claims', true)::json->>'role' IN ('editor', 'admin', 'service_role')
            OR current_setting('role', true) = 'postgres'
        );
    END IF;
END $$;
