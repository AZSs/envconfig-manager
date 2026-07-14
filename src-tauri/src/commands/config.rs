use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::backup;
use crate::utils::fs_atomic;
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

/// 可注入备份目录与元数据路径的写入实现(便于测试)。
///
/// 流程:若目标文件已存在,先创建备份;再原子写入新内容。
pub(crate) fn write_config_file_into(
    backup_dir: &Path,
    meta_path: &Path,
    path: &str,
    content: &str,
) -> Result<(), String> {
    let target = Path::new(path);

    // 目标已存在则先备份,保证可回滚
    if target.exists() {
        backup::backup_file_into(backup_dir, meta_path, path, "写入前自动备份")?;
    }

    // 原子写入:写临时文件 → rename,避免半截写入损坏用户配置
    fs_atomic::write_text_atomic(target, content)
}

/// 写入配置文件内容
///
/// 写入前自动备份(若文件已存在),并以原子方式覆盖,避免半截写入。
#[tauri::command]
pub fn write_config_file(path: String, content: String) -> Result<(), String> {
    let backup_dir = backup_dir_default();
    let meta_path = meta_path_default();
    write_config_file_into(&backup_dir, &meta_path, &path, &content)
}

/// 使配置文件生效（source）
#[tauri::command]
pub fn apply_config_file(path: String) -> Result<String, String> {
    platform::source_config_file(&path)
}

pub(crate) fn backup_dir_default() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups")
}

pub(crate) fn meta_path_default() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_creates_file_when_missing_without_backup() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");
        let target = dir.path().join(".zshrc");

        write_config_file_into(&backup_dir, &meta_path, target.to_str().unwrap(), "NEW").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW");
        // 文件原本不存在,不应产生备份
        assert!(!meta_path.exists());
    }

    #[test]
    fn write_backs_up_existing_before_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");
        let target = dir.path().join(".zshrc");

        fs::write(&target, "OLD CONTENT").unwrap();
        write_config_file_into(&backup_dir, &meta_path, target.to_str().unwrap(), "NEW CONTENT").unwrap();

        // 目标已是新内容
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW CONTENT");

        // 备份保留了旧内容
        let snapshots = backup::load_snapshots_from(&meta_path);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(fs::read_to_string(&snapshots[0].file_path).unwrap(), "OLD CONTENT");
    }

    #[test]
    fn write_replaces_content_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");
        let target = dir.path().join(".bashrc");

        fs::write(&target, "A").unwrap();
        write_config_file_into(&backup_dir, &meta_path, target.to_str().unwrap(), "B").unwrap();
        write_config_file_into(&backup_dir, &meta_path, target.to_str().unwrap(), "C").unwrap();

        assert_eq!(fs::read_to_string(&target).unwrap(), "C");
        // 两次写入产生两次备份
        let snapshots = backup::load_snapshots_from(&meta_path);
        assert_eq!(snapshots.len(), 2);
    }
}
