-- Add migration script here
-- Create Products Table
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    brand VARCHAR(50) NOT NULL,
    description VARCHAR(500),
    price NUMERIC(10, 2) NOT NULL,
    stock INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_name ON products(name);
CREATE INDEX idx_products_brand ON products(brand);
