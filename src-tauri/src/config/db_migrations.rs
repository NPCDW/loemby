use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: r#"
CREATE TABLE "emby_server" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
	"base_url" VARCHAR(255) NOT NULL,
	"username" VARCHAR(255),
	"password" VARCHAR(255),
	"server_name" VARCHAR(255),
	"server_id" VARCHAR(255),
	"auth_token" VARCHAR(255),
	"user_id" VARCHAR(255),
	"client" VARCHAR(255) NOT NULL,
	"device" VARCHAR(255) NOT NULL,
	"device_id" VARCHAR(255) NOT NULL,
	"client_version" VARCHAR(255) NOT NULL,
	"user_agent" VARCHAR(255) NOT NULL,
	"order_by" INTEGER NOT NULL,
	"browse_proxy_id" VARCHAR(255) NOT NULL,
	"play_proxy_id" VARCHAR(255) NOT NULL,
	"last_playback_time" TIMESTAMP DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
  "keep_alive_days" int NOT NULL DEFAULT 0,
  "disabled" int NOT NULL DEFAULT 0
);
CREATE INDEX emby_server_order on emby_server (order_by);
CREATE TABLE "global_config" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
	"config_key" VARCHAR(255) NOT NULL,
	"config_value" VARCHAR(255) NOT NULL
);
CREATE UNIQUE INDEX global_config_key on global_config (config_key);
CREATE TABLE "proxy_server" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
	"name" VARCHAR(255) NOT NULL,
	"proxy_type" VARCHAR(255) NOT NULL,
	"addr" VARCHAR(255) NOT NULL,
	"username" VARCHAR(255),
	"password" VARCHAR(255)
);
CREATE TABLE "emby_line" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
	"name" VARCHAR(255) NOT NULL,
    "emby_server_id" CHAR(36) NOT NULL,
	"emby_server_name" VARCHAR(255) NOT NULL,
	"base_url" VARCHAR(255) NOT NULL,
	"browse_proxy_id" VARCHAR(255) NOT NULL,
	"play_proxy_id" VARCHAR(255) NOT NULL,
	"in_use" int NOT NULL DEFAULT 0
);
CREATE INDEX emby_line_emby_server_id ON emby_line (emby_server_id);
"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "emby_icon_library",
            sql: r#"
CREATE TABLE "emby_icon_library" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
  "name" VARCHAR(255) NOT NULL,
	"url" VARCHAR(255) NOT NULL
);
ALTER TABLE emby_server ADD COLUMN icon_url VARCHAR(255);
			"#,
			kind: MigrationKind::Up,
		},
        Migration {
            version: 3,
            description: "emby_line_id",
            sql: r#"
ALTER TABLE emby_server ADD COLUMN line_id CHAR(36);
			"#,
			kind: MigrationKind::Up,
		},
        Migration {
            version: 4,
            description: "play_history",
            sql: r#"
CREATE TABLE "play_history" (
  "id" CHAR(36) NOT NULL PRIMARY KEY,
  "create_time" TIMESTAMP NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
	"update_time" TIMESTAMP DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
    "emby_server_id" CHAR(36) NOT NULL,
	"emby_server_name" VARCHAR(255) NOT NULL,
	"item_type" VARCHAR(255) NOT NULL,
	"item_id" VARCHAR(255) NOT NULL,
	"item_name" VARCHAR(255) NOT NULL,
	"series_id" VARCHAR(255),
	"series_name" VARCHAR(255),
  "played_duration" INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX play_history_key on play_history (emby_server_id, item_id);
			"#,
			kind: MigrationKind::Up,
		},
        Migration {
            version: 5,
            description: "play_history",
            sql: r#"
ALTER TABLE play_history ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
CREATE INDEX play_history_pinned_update_time ON play_history (pinned, update_time);
			"#,
			kind: MigrationKind::Up,
		},
    ]
}