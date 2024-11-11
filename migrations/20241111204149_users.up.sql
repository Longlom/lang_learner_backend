-- Add up migration script here
CREATE TYPE language AS ENUM ('VN', 'CH');
CREATE TABLE
    IF NOT EXISTS users (
        "id" SERIAL PRIMARY KEY,
        "password" character(100) NOT NULL,
        "login" character(100) NOT NULL,
        "username" character varying(150) NOT NULL,
        "language" language NOT NULL
    );