BEGIN;
INSERT INTO admin_users (id, login, hashed_password, two_fa_secret, is_system, created_by)
VALUES (1, 'system', '', '', true, 1);
COMMIT;