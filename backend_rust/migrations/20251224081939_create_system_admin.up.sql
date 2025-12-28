BEGIN;
INSERT INTO admin_users (login, hashed_password, two_fa_secret, is_system, created_by)
VALUES ('system', '', '', true, 1);
COMMIT;