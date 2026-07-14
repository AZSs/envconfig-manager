use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::fs_atomic;

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

const MAX_SNAPSHOTS_DEFAULT: usize = 50;

fn get_backup_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups")
}

fn get_meta_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".envconfig").join("backups.json")
}

pub(crate) fn load_snapshots_from(meta_path: &Path) -> Vec<BackupSnapshot> {
    if meta_path.exists() {
        let content = fs::read_to_string(meta_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_snapshots_to(meta_path: &Path, snapshots: &[BackupSnapshot]) -> Result<(), String> {
    fs_atomic::write_json_atomic(meta_path, snapshots)
}

fn load_snapshots() -> Vec<BackupSnapshot> {
    load_snapshots_from(&get_meta_path())
}

fn save_snapshots(snapshots: &[BackupSnapshot]) -> Result<(), String> {
    save_snapshots_to(&get_meta_path(), snapshots)
}

/// 自动清理超出数量限制的备份(删除最旧)。
fn auto_cleanup(snapshots: &mut Vec<BackupSnapshot>, max_snapshots: usize) {
    while snapshots.len() > max_snapshots {
        if let Some(oldest) = snapshots.first() {
            let _ = fs::remove_file(&oldest.file_path);
        }
        snapshots.remove(0);
    }
}

/// 可注入备份目录与元数据路径的核心备份实现(便于测试)。
///
/// 将 `source_path` 复制到 `backup_dir`,并在 `meta_path` 登记一条快照。
pub(crate) fn backup_file_into(
    backup_dir: &Path,
    meta_path: &Path,
    source_path: &str,
    remark: &str,
) -> Result<BackupSnapshot, String> {
    let source = PathBuf::from(source_path);
    if !source.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }

    fs::create_dir_all(backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let now = chrono::Local::now();
    let source_name = source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    // 用 uuid 保证文件名唯一,避免同秒多次备份互相覆盖。
    let short_id = &uuid::Uuid::new_v4().to_string()[..8];
    let backup_name = format!("{}_{}_{}.bak", source_name, now.format("%Y%m%d_%H%M%S"), short_id);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(&source, &backup_path).map_err(|e| format!("备份文件失败: {}", e))?;

    let file_size = fs::metadata(&backup_path).map(|m| m.len()).unwrap_or(0);

    let snapshot = BackupSnapshot {
        id: uuid::Uuid::new_v4().to_string(),
        file_name: backup_name,
        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        remark: remark.to_string(),
        file_size,
        file_path: backup_path.to_string_lossy().to_string(),
    };

    let mut snapshots = load_snapshots_from(meta_path);
    snapshots.push(snapshot.clone());
    auto_cleanup(&mut snapshots, MAX_SNAPSHOTS_DEFAULT);
    save_snapshots_to(meta_path, &snapshots)?;

    Ok(snapshot)
}

/// 用默认目录(~/.envconfig)创建备份。供其它模块与 create_backup 命令复用。
pub fn backup_file_internal(source_path: &str, remark: &str) -> Result<BackupSnapshot, String> {
    backup_file_into(&get_backup_dir(), &get_meta_path(), source_path, remark)
}

/// 创建备份
#[tauri::command]
pub fn create_backup(source_path: String, remark: String) -> Result<BackupSnapshot, String> {
    backup_file_internal(&source_path, &remark)
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
    let _ = backup_file_internal(&target_path, "回滚前自动备份");

    fs::copy(&snapshot.file_path, &target_path)
        .map_err(|e| format!("回滚失败: {}", e))?;

    // 生效配置
    let _ = crate::utils::platform::source_config_file(&target_path);

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
        max_snapshots: MAX_SNAPSHOTS_DEFAULT,
        max_size_mb: 100,
        backup_dir: get_backup_dir().to_string_lossy().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_file_into_copies_source_and_records_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");

        // 准备源文件
        let source = dir.path().join(".zshrc");
        fs::write(&source, "export FOO=bar\n").unwrap();

        let snap = backup_file_into(&backup_dir, &meta_path, source.to_str().unwrap(), "测试").unwrap();

        // 备份文件存在且内容一致
        assert!(Path::new(&snap.file_path).exists());
        assert_eq!(fs::read_to_string(&snap.file_path).unwrap(), "export FOO=bar\n");
        assert_eq!(snap.remark, "测试");

        // 元数据登记了一条记录
        let snapshots = load_snapshots_from(&meta_path);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, snap.id);
    }

    #[test]
    fn backup_file_into_appends_to_existing_snapshots() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");

        let source = dir.path().join(".zshrc");
        fs::write(&source, "v1\n").unwrap();
        backup_file_into(&backup_dir, &meta_path, source.to_str().unwrap(), "first").unwrap();

        fs::write(&source, "v2\n").unwrap();
        backup_file_into(&backup_dir, &meta_path, source.to_str().unwrap(), "second").unwrap();

        let snapshots = load_snapshots_from(&meta_path);
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn backup_file_into_fails_when_source_missing() {
        let dir = tempfile::tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let meta_path = dir.path().join("backups.json");
        let missing = dir.path().join("nope");

        let result = backup_file_into(&backup_dir, &meta_path, missing.to_str().unwrap(), "x");
        assert!(result.is_err());
    }
}
