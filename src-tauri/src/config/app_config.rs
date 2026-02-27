use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_inline_default::serde_inline_default;

use crate::util::file_util;
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde_inline_default("info".to_string())]
    pub log_level: String,
    #[serde_inline_default("sqlite".to_string())]
    pub database_type: String,
    #[serde_inline_default("".to_string())]
    pub database_url: String,
    #[serde_inline_default("".to_string())]
    pub dev_private_key: String,
}

const APP_CONFIG_PATH: &'static str = "config/app-config.json";
const RESOURCES_CONFIG_PATH: &'static str = "resources/config/app-config.default.json";

pub fn get_config(app: &tauri::App) -> anyhow::Result<Config> {
    let config_dir = app.path().resolve("", tauri::path::BaseDirectory::AppConfig)?;
    let config_path = config_dir.join(APP_CONFIG_PATH);
    if !config_path.exists() {
        file_util::mkdir(config_path.parent().unwrap())?;
        let resource_path = app.path().resolve(
            RESOURCES_CONFIG_PATH,
            tauri::path::BaseDirectory::Resource,
        )?;
        file_util::copy(&resource_path, &config_path)?;
    }
    let content = file_util::read_file(&config_path)?;
    anyhow::Ok(serde_json::from_str(&content)?)
}

/// Ed25519 公钥（硬编码），与 dev_private_key 配对时开启开发模式
const DEV_PUBLIC_KEY_BYTES: [u8; 32] = [211, 19, 10, 106, 133, 72, 140, 20, 67, 126, 183, 86, 89, 148, 93, 121, 133, 221, 33, 40, 58, 85, 224, 198, 109, 108, 201, 148, 34, 216, 122, 219];

/// 用于签名验证的固定消息
const DEV_MODE_MESSAGE: &[u8] = b"loemby_dev_mode";

/// 判断是否处于开发模式：使用 dev_private_key_hex 对固定消息签名，
/// 再用硬编码公钥验证，若通过则认为是开发者授权。
pub fn is_dev_mode(dev_private_key_hex: &str) -> bool {
    if dev_private_key_hex.is_empty() {
        return false;
    }
    let key_bytes = match hex::decode(dev_private_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let key_array: [u8; 32] = match key_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let signing_key = SigningKey::from_bytes(&key_array);
    let signature: Signature = signing_key.sign(DEV_MODE_MESSAGE);
    let verifying_key = match VerifyingKey::from_bytes(&DEV_PUBLIC_KEY_BYTES) {
        Ok(k) => k,
        Err(_) => return false,
    };
    verifying_key.verify(DEV_MODE_MESSAGE, &signature).is_ok()
}
