-- Keep existing accounts. Only the selected account must be unique.
DROP INDEX IF EXISTS minecraft_users_active;
CREATE UNIQUE INDEX minecraft_users_active ON minecraft_users(active) WHERE active = 1;
