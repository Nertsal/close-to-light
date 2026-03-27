-- Link steam id to user account.
ALTER TABLE user_linked_accounts
ADD COLUMN steam BLOB;
