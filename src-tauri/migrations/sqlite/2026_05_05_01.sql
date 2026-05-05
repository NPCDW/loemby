-- 1. 创建新表，使用正确的字段类型
CREATE TABLE "emby_server_new" (
    "id" CHAR(36) NOT NULL PRIMARY KEY,
    "create_time" TEXT NOT NULL,
    "base_url" VARCHAR(255) NOT NULL,
    "username" VARCHAR(255),
    "password" VARCHAR(255),
    "server_name" VARCHAR(255),
    "server_id" VARCHAR(255),
    "auth_token" TEXT,   -- 修改此字段
    "user_id" VARCHAR(255),
    "client" VARCHAR(255) NOT NULL,
    "device" VARCHAR(255) NOT NULL,
    "device_id" VARCHAR(255) NOT NULL,
    "client_version" VARCHAR(255) NOT NULL,
    "user_agent" VARCHAR(255) NOT NULL,
    "order_by" INTEGER NOT NULL,
    "browse_proxy_id" VARCHAR(255) NOT NULL,
    "play_proxy_id" VARCHAR(255) NOT NULL,
    "last_playback_time" TEXT,
    "keep_alive_days" int NOT NULL DEFAULT 0,
    "disabled" int NOT NULL DEFAULT 0,
    "icon_url" VARCHAR(255),
    "line_id" CHAR(36)
);

-- 2. 复制原表数据
INSERT INTO emby_server_new (id, create_time, base_url, username, password, server_name, server_id, auth_token, user_id, client, device, device_id, client_version, user_agent, order_by, browse_proxy_id, play_proxy_id, last_playback_time, keep_alive_days, disabled, icon_url, line_id)
SELECT id, create_time, base_url, username, password, server_name, server_id, auth_token, user_id, client, device, device_id, client_version, user_agent, order_by, browse_proxy_id, play_proxy_id, last_playback_time, keep_alive_days, disabled, icon_url, line_id
FROM emby_server;

-- 3. 删除原表
DROP TABLE emby_server;

-- 4. 重命名新表
ALTER TABLE emby_server_new RENAME TO emby_server;

-- 5. 重建索引和触发器（如果有）
CREATE INDEX emby_server_order on emby_server (order_by);