-- Convert role columns from VARCHAR + CHECK to native PostgreSQL enum types.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role_enum') THEN
        CREATE TYPE user_role_enum AS ENUM ('admin', 'instructor', 'ta', 'student');
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'class_membership_role_enum') THEN
        CREATE TYPE class_membership_role_enum AS ENUM ('ta', 'student');
    END IF;
END $$;

ALTER TABLE users
    ALTER COLUMN user_role DROP DEFAULT;

ALTER TABLE users
    ALTER COLUMN user_role TYPE user_role_enum
        USING user_role::user_role_enum;

ALTER TABLE users
    ALTER COLUMN user_role SET DEFAULT 'student'::user_role_enum;

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_users_user_role;

ALTER TABLE class_memberships
    ALTER COLUMN role TYPE class_membership_role_enum
        USING role::class_membership_role_enum;

ALTER TABLE class_memberships
    DROP CONSTRAINT IF EXISTS chk_class_memberships_role;
