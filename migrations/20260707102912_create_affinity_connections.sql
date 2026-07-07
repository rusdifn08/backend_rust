CREATE TABLE IF NOT EXISTS affinity_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    affinity_type VARCHAR(50) NOT NULL, -- 'partner', 'bro', 'bestie', 'confidant'
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'accepted'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(requester_id, receiver_id)
);
