-- Grade-O-Matic seed data aligned with current migrations/schema.
-- Includes: users (with enum user_role), user_auth, devices, classes,
-- class_memberships (enum role), and assignments.

BEGIN;

-- Clear in FK-safe order so the seed can be rerun.
TRUNCATE TABLE
    assignments,
    class_memberships,
    classes,
    devices,
    user_auth,
    uploaded_files,
    users
    RESTART IDENTITY CASCADE;

-- Users
INSERT INTO users (id, username, email, user_role, created_by, created_at, modified_by, modified_at)
VALUES ('00000000-0000-0000-0000-000000000001', 'admin01', 'admin01@example.com', 'admin', NULL, NOW(), NULL, NOW()),
       ('00000000-0000-0000-0000-000000000002', 'instructor01', 'instructor01@example.com', 'instructor', NULL, NOW(),
        NULL, NOW()),
       ('00000000-0000-0000-0000-000000000003', 'ta01', 'ta01@example.com', 'ta', NULL, NOW(), NULL, NOW()),
       ('00000000-0000-0000-0000-000000000004', 'student01', 'student01@example.com', 'student', NULL, NOW(), NULL,
        NOW()),
       ('00000000-0000-0000-0000-000000000005', 'student02', 'student02@example.com', 'student', NULL, NOW(), NULL,
        NOW()),
       ('00000000-0000-0000-0000-000000000021', 'apitest01', 'apitest01@example.com', 'student', NULL, NOW(), NULL,
        NOW());

-- User auth
-- Login credentials:
--   client_id: apitest01
--   client_secret: test_password
--   client_id: admin01
--   client_secret: test_password
--   client_id: instructor01
--   client_secret: test_password
--   client_id: student01
--   client_secret: test_password
INSERT INTO user_auth (user_id, password_hash, created_at, modified_at)
VALUES ('00000000-0000-0000-0000-000000000001',
        '$argon2id$v=19$m=19456,t=2,p=1$XBFwBY52C9SpzkxON1OTLg$djDqZQvzxFKc9HOCWyZfKy+RlFTs0BJFSkcw/Tos14c',
        NOW(),
        NOW()),
       ('00000000-0000-0000-0000-000000000002',
        '$argon2id$v=19$m=19456,t=2,p=1$XBFwBY52C9SpzkxON1OTLg$djDqZQvzxFKc9HOCWyZfKy+RlFTs0BJFSkcw/Tos14c',
        NOW(),
        NOW()),
       ('00000000-0000-0000-0000-000000000021',
        '$argon2id$v=19$m=19456,t=2,p=1$XBFwBY52C9SpzkxON1OTLg$djDqZQvzxFKc9HOCWyZfKy+RlFTs0BJFSkcw/Tos14c',
        NOW(),
        NOW()),
       ('00000000-0000-0000-0000-000000000004',
        '$argon2id$v=19$m=19456,t=2,p=1$XBFwBY52C9SpzkxON1OTLg$djDqZQvzxFKc9HOCWyZfKy+RlFTs0BJFSkcw/Tos14c',
        NOW(),
        NOW());

-- Devices
INSERT INTO devices (id, user_id, name, status, device_os, registered_at, created_by, created_at, modified_by,
                     modified_at)
VALUES ('10000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'admin-macbook', 'active',
        'iOS', NOW(), '00000000-0000-0000-0000-000000000001', NOW(), '00000000-0000-0000-0000-000000000001', NOW()),
       ('10000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000002', 'instructor-ipad', 'active',
        'iOS', NOW(), '00000000-0000-0000-0000-000000000002', NOW(), '00000000-0000-0000-0000-000000000002', NOW()),
       ('10000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000003', 'ta-android-phone', 'inactive',
        'Android', NOW(), '00000000-0000-0000-0000-000000000003', NOW(), '00000000-0000-0000-0000-000000000003', NOW()),
       ('10000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000004', 'student-laptop', 'pending',
        'Android', NOW(), '00000000-0000-0000-0000-000000000004', NOW(), '00000000-0000-0000-0000-000000000004', NOW());

-- Classes
INSERT INTO classes (id, title, description, term, owner_id, created_by, created_at, modified_by, modified_at)
VALUES ('20000000-0000-0000-0000-000000000001',
        'Intro to Rust',
        'Foundations of Rust programming for backend development.',
        'Spring 2026',
        '00000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000002',
        NOW(),
        '00000000-0000-0000-0000-000000000002',
        NOW()),
       ('20000000-0000-0000-0000-000000000002',
        'Web Systems',
        'Server-side web architecture and APIs.',
        'Spring 2026',
        '00000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000002',
        NOW(),
        '00000000-0000-0000-0000-000000000002',
        NOW());

-- Class memberships
INSERT INTO class_memberships (id, class_id, user_id, role, created_at, modified_at)
VALUES ('30000000-0000-0000-0000-000000000001', '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000003', 'ta', NOW(), NOW()),
       ('30000000-0000-0000-0000-000000000002', '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000004', 'student', NOW(), NOW()),
       ('30000000-0000-0000-0000-000000000003', '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000005', 'student', NOW(), NOW()),
       ('30000000-0000-0000-0000-000000000004', '20000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000004', 'student', NOW(), NOW());

-- Assignments
INSERT INTO assignments (id, class_id, title, description, due_at, deadline_type, points, created_by, created_at,
                         modified_by, modified_at)
VALUES ('40000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        'Ownership and Borrowing',
        'Practice references, lifetimes, and ownership transfer.',
        NOW() + INTERVAL '7 days',
        'soft_deadline',
        100,
        '00000000-0000-0000-0000-000000000002',
        NOW(),
        '00000000-0000-0000-0000-000000000002',
        NOW()),
       ('40000000-0000-0000-0000-000000000002',
        '20000000-0000-0000-0000-000000000001',
        'Error Handling',
        'Build a CLI with robust Result-based error handling.',
        NOW() + INTERVAL '14 days',
        'soft_deadline',
        50,
        '00000000-0000-0000-0000-000000000002',
        NOW(),
        '00000000-0000-0000-0000-000000000002',
        NOW()),
       ('40000000-0000-0000-0000-000000000003',
        '20000000-0000-0000-0000-000000000002',
        'HTTP Middleware',
        'Implement request logging and auth middleware in Axum.',
        NOW() + INTERVAL '10 days',
        'soft_deadline',
        100,
        '00000000-0000-0000-0000-000000000002',
        NOW(),
        '00000000-0000-0000-0000-000000000002',
        NOW());

COMMIT;
