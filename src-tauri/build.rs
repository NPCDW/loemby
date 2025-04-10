use std::path::PathBuf;

const VERSION: &'static str = "0.10.0";

fn main() {
    modify_version();
    tauri_build::build()
}

fn modify_version() {
    let current_dir = std::env::current_dir().unwrap();
    println!("当前所在目录: {:?}", current_dir);
    let parent_dir = current_dir.parent().unwrap();
    
    let file = parent_dir.join("package.json");
    let regex_m = regex::Regex::new(r#"\"version\": \"(.*)\""#).unwrap();
    let replace_content = format!(r#""version": "{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);

    let file = parent_dir.join(".env");
    let regex_m = regex::Regex::new(r#"VITE_APP_VERSION=(.*)"#).unwrap();
    let replace_content = format!(r#"VITE_APP_VERSION={}"#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);
    
    let file = current_dir.join("Cargo.toml");
    let regex_m = regex::Regex::new(r#"version = \"(.*)\""#).unwrap();
    let replace_content = format!(r#"version = "{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);
    
    let file = current_dir.join("tauri.beta.conf.json");
    let regex_m = regex::Regex::new(r#"\"version\": \"(.*)\""#).unwrap();
    let replace_content = format!(r#""version": "{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);
    let regex_m = regex::Regex::new(r#"\"userAgent\": \"loemby/(.*)\""#).unwrap();
    let replace_content = format!(r#""userAgent": "loemby/{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);

    let file = current_dir.join("tauri.conf.json");
    let regex_m = regex::Regex::new(r#"\"version\": \"(.*)\""#).unwrap();
    let replace_content = format!(r#""version": "{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);
    let regex_m = regex::Regex::new(r#"\"userAgent\": \"loemby/(.*)\""#).unwrap();
    let replace_content = format!(r#""userAgent": "loemby/{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);

    let file = current_dir.join("tauri.dev.conf.json");
    let regex_m = regex::Regex::new(r#"\"version\": \"(.*)\""#).unwrap();
    let replace_content = format!(r#""version": "{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);
    let regex_m = regex::Regex::new(r#"\"userAgent\": \"loemby/(.*)\""#).unwrap();
    let replace_content = format!(r#""userAgent": "loemby/{}""#, VERSION);
    replace_file_content(&file, regex_m, &replace_content);

}

fn replace_file_content(file: &PathBuf, regex_m: regex::Regex, replace_content: &str) {
    println!("开始替换文件: {:?} {:?}", file, regex_m);
    let mut content = std::fs::read_to_string(&file).unwrap();
    if VERSION != &regex_m.captures(&content).unwrap()[1] {
        content = regex_m.replace(&content, replace_content).to_string();
        std::fs::write(&file, &content).unwrap();
    };
}