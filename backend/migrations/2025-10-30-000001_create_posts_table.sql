-- Add posts table
CREATE TABLE IF NOT EXISTS posts (
    id uuid PRIMARY KEY,
    title text NOT NULL,
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
