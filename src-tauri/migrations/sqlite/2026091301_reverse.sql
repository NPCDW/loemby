CREATE TABLE "reverse_proxy_server" (
    "id" CHAR(36) NOT NULL PRIMARY KEY,
    "create_time" TEXT NOT NULL,
    "name" VARCHAR(255) NOT NULL,
    "url" VARCHAR(255) NOT NULL
);

ALTER TABLE emby_server ADD COLUMN reverse_proxy_id VARCHAR(255) NOT NULL DEFAULT 'no';

ALTER TABLE emby_line ADD COLUMN reverse_proxy_id VARCHAR(255) NOT NULL DEFAULT 'no';
