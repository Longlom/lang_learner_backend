-- Add up migration script here
CREATE TYPE jwt_type_enum AS ENUM ('refresh', 'access');
CREATE TABLE
    IF NOT EXISTS jwt_token (
        "id" SERIAL PRIMARY KEY,
        "user_id" SERIAL,
        "created_at" timestamp with time zone NOT NULL DEFAULT now (),
        "expires_in" timestamp NOT NULL,
        "active" boolean NOT NULL DEFAULT FALSE,
        "type" jwt_type_enum NOT NULL
    );