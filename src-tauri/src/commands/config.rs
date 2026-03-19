use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils::platform;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub path: String,
    pub name: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub content: String,
    #[serde(rename = "shellType")]
    pub shell_type: String,
}

/// 扫描系统中的配置文件
#[tauri::command]
pub fn scan_config_files() -> Result<Vec<ConfigFile>, String> {
    let paths = platform::get_config_file_paths();
    let mut files = Vec::new();

    for path in paths {
        let path_buf = PathBuf::from(&path);
        if path_buf.exists() {
            let metadata = fs::metadata(&path_buf).map_err(|e| e.to_string())?;
            let modified = metadata
                .modified()
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_default();

            let name = path_buf
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let shell_type = platform::detect_shell_type(&name);

            files.push(ConfigFile {
                path: path.clone(),
                name,
                last_modified: modified,
                content: String::new(), // 扫描时不读内容
                shell_type,
            });
        }
    }

    Ok(files)
}

/// 读取配置文件内容
#[tauri::command]
pub fn read_config_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 写入配置文件内容
#[tauri::command]
pub fn write_config_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, &content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 使配置文件生效（source）
#[tauri::command]
pub fn apply_config_file(path: String) -> Result<String, String> {
    platform::source_config_file(&path)
}
