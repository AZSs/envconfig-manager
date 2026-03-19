use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils::platform;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupSnapshot {
    pub id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub timestamp: String,
    pub remark: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupConfig {
    #[serde(rename = "maxSnapshots")]
    pub max_snapshots: usize,
    #[serde(rename = "maxSizeMB")]
    pub max_size_mb: u64,
    #[serde(rename = "backupDir")]
    pub backup_dir: String,
}

fn get_backup_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups")
}

fn get_meta_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups.json")
}

fn load_snapshots() -> Vec<BackupSnapshot> {
    let meta_path = get_meta_path();
    if meta_path.exists() {
        let content = fs::read_to_string(&meta_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_snapshots(snapshots: &[BackupSnapshot]) -> Result<(), String> {
    let meta_path = get_meta_path();
    if let Some(parent) = meta_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(snapshots).map_err(|e| e.to_string())?;
    fs::write(&meta_path, content).map_err(|e| e.to_string())
}

/// 自动清理超出限制的备份
fn auto_cleanup(snapshots: &mut Vec<BackupSnapshot>, max_snapshots: usize) {
    while snapshots.len() > max_snapshots {
        if let Some(oldest) = snapshots.first() {
            let _ = fs::remove_file(&oldest.file_path);
        }
        snapshots.remove(0);
    }
}

/// 创建备份
#[tauri::command]
pub fn create_backup(source_path: String, remark: String) -> Result<BackupSnapshot, String> {
    let backup_dir = get_backup_dir();
    fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let source_name = PathBuf::from(&source_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let backup_name = format!("{}_{}.bak", source_name, now.format("%Y%m%d_%H%M%S"));
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(&source_path, &backup_path).map_err(|e| format!("备份文件失败: {}", e))?;

    let file_size = fs::metadata(&backup_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let snapshot = BackupSnapshot {
        id,
        file_name: backup_name,
        timestamp,
        remark,
        file_size,
        file_path: backup_path.to_string_lossy().to_string(),
    };

    let mut snapshots = load_snapshots();
    snapshots.push(snapshot.clone());
    auto_cleanup(&mut snapshots, 50);
    save_snapshots(&snapshots)?;

    Ok(snapshot)
}

/// 列出所有备份
#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupSnapshot>, String> {
    Ok(load_snapshots())
}

/// 回滚备份
#[tauri::command]
pub fn restore_backup(backup_id: String, target_path: String) -> Result<(), String> {
    let snapshots = load_snapshots();
    let snapshot = snapshots
        .iter()
        .find(|s| s.id == backup_id)
        .ok_or("备份快照不存在")?;

    // 回滚前先备份当前文件
    let _ = create_backup(target_path.clone(), "回滚前自动备份".to_string());

    fs::copy(&snapshot.file_path, &target_path)
        .map_err(|e| format!("回滚失败: {}", e))?;

    // 生效配置
    let _ = platform::source_config_file(&target_path);

    Ok(())
}

/// 删除备份
#[tauri::command]
pub fn delete_backup(backup_id: String) -> Result<(), String> {
    let mut snapshots = load_snapshots();
    if let Some(pos) = snapshots.iter().position(|s| s.id == backup_id) {
        let snapshot = snapshots.remove(pos);
        let _ = fs::remove_file(&snapshot.file_path);
        save_snapshots(&snapshots)?;
        Ok(())
    } else {
        Err("备份快照不存在".to_string())
    }
}

/// 获取备份配置
#[tauri::command]
pub fn get_backup_config() -> Result<BackupConfig, String> {
    Ok(BackupConfig {
        max_snapshots: 50,
        max_size_mb: 100,
        backup_dir: get_backup_dir().to_string_lossy().to_string(),
    })
}
