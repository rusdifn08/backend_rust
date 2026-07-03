CREATE TYPE item_category AS ENUM ('Consumable', 'Emblem', 'Pet', 'Theme', 'UICustom');

CREATE TABLE IF NOT EXISTS shop_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price INT NOT NULL,
    category item_category NOT NULL,
    image_url VARCHAR(255),
    max_purchases INT, -- NULL means unlimited
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    item_id UUID REFERENCES shop_items(id) ON DELETE CASCADE,
    quantity INT NOT NULL DEFAULT 1,
    is_equipped BOOLEAN NOT NULL DEFAULT false,
    acquired_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, item_id)
);
