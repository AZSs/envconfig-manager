use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils::platform;

// ======================== 数据结构 ========================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigProfileEntry {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVarEntry {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub entries: Vec<ConfigProfileEntry>,
    #[serde(rename = "envVars")]
    pub env_vars: Vec<EnvVarEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileDiffItem {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "currentContent")]
    pub current_content: String,
    #[serde(rename = "profileContent")]
    pub profile_content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileDiffResult {
    pub diffs: Vec<ProfileDiffItem>,
    #[serde(rename = "hasUnsavedChanges")]
    pub has_unsaved_changes: bool,
}

// ======================== 辅助函数 ========================

/// 获取配置集目录，不存在则自动创建
fn get_profiles_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".envconfig").join("profiles");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

/// 获取 profiles.json 路径
fn get_profiles_path() -> PathBuf {
    get_profiles_dir().join("profiles.json")
}

/// 加载所有配置集
fn load_profiles() -> Vec<ConfigProfile> {
    let path = get_profiles_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// 保存所有配置集
fn save_profiles(profiles: &[ConfigProfile]) -> Result<(), String> {
    let path = get_profiles_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置集目录失败: {}", e))?;
    }
    let content = serde_json::to_string_pretty(profiles).map_err(|e| format!("序列化配置集失败: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("保存配置集失败: {}", e))
}

/// 为文件创建简单备份（复制到 ~/.envconfig/backups/ 并添加时间戳后缀）
fn create_simple_backup(file_path: &str) -> Result<(), String> {
    let source = PathBuf::from(file_path);
    if !source.exists() {
        return Ok(()); // 文件不存在则跳过备份
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let backup_dir = home.join(".envconfig").join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let file_name = source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("{}_{}.bak", file_name, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(&source, &backup_path).map_err(|e| format!("备份文件失败: {}", e))?;
    Ok(())
}

/// 获取当前时间字符串
fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ======================== Tauri 命令 ========================

/// 列出所有配置集
#[tauri::command]
pub fn list_profiles() -> Result<Vec<ConfigProfile>, String> {
    Ok(load_profiles())
}

/// 创建配置集
#[tauri::command]
pub fn create_profile(
    name: String,
    description: String,
    entries: Vec<ConfigProfileEntry>,
    env_vars: Vec<EnvVarEntry>,
) -> Result<ConfigProfile, String> {
    let now = now_string();
    let profile = ConfigProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description,
        active: false,
        created_at: now.clone(),
        updated_at: now,
        entries,
        env_vars,
    };

    let mut profiles = load_profiles();
    profiles.push(profile.clone());
    save_profiles(&profiles)?;

    Ok(profile)
}

/// 更新配置集
#[tauri::command]
pub fn update_profile(
    id: String,
    name: String,
    description: String,
    entries: Vec<ConfigProfileEntry>,
    env_vars: Vec<EnvVarEntry>,
) -> Result<ConfigProfile, String> {
    let mut profiles = load_profiles();
    let profile = profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("配置集不存在: {}", id))?;

    profile.name = name;
    profile.description = description;
    profile.entries = entries;
    profile.env_vars = env_vars;
    profile.updated_at = now_string();

    let updated = profile.clone();
    save_profiles(&profiles)?;

    Ok(updated)
}

/// 删除配置集
#[tauri::command]
pub fn delete_profile(id: String) -> Result<(), String> {
    let mut profiles = load_profiles();
    let original_len = profiles.len();
    profiles.retain(|p| p.id != id);

    if profiles.len() == original_len {
        return Err(format!("配置集不存在: {}", id));
    }

    save_profiles(&profiles)?;
    Ok(())
}

/// 应用配置集：将配置内容写入对应文件，设置环境变量，并生效配置
#[tauri::command]
pub fn apply_profile(id: String) -> Result<Vec<String>, String> {
    let profiles = load_profiles();
    let profile = profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("配置集不存在: {}", id))?;

    let mut modified_paths: Vec<String> = Vec::new();

    // 写入配置文件
    for entry in &profile.entries {
        // 创建备份
        create_simple_backup(&entry.file_path)?;

        // 确保父目录存在
        let target = PathBuf::from(&entry.file_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
        }

        // 写入内容
        fs::write(&target, &entry.content)
            .map_err(|e| format!("写入文件失败 {}: {}", entry.file_path, e))?;

        modified_paths.push(entry.file_path.clone());
    }

    // 设置环境变量
    for var in &profile.env_vars {
        platform::set_env_variable(&var.name, &var.value, "user")
            .map_err(|e| format!("设置环境变量失败 {}={}: {}", var.name, var.value, e))?;
    }

    // 生效已修改的配置文件
    for path in &modified_paths {
        let _ = platform::source_config_file(path);
    }

    Ok(modified_paths)
}

/// 对比配置集与当前文件内容的差异
#[tauri::command]
pub fn diff_profile(id: String) -> Result<ProfileDiffResult, String> {
    let profiles = load_profiles();
    let profile = profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("配置集不存在: {}", id))?;

    let mut diffs: Vec<ProfileDiffItem> = Vec::new();
    let mut has_unsaved_changes = false;

    for entry in &profile.entries {
        let current_content = fs::read_to_string(&entry.file_path).unwrap_or_default();

        if current_content != entry.content {
            has_unsaved_changes = true;
        }

        diffs.push(ProfileDiffItem {
            file_path: entry.file_path.clone(),
            current_content,
            profile_content: entry.content.clone(),
        });
    }

    Ok(ProfileDiffResult {
        diffs,
        has_unsaved_changes,
    })
}

/// 导出配置集为 JSON 字符串
#[tauri::command]
pub fn export_profile(id: String) -> Result<String, String> {
    let profiles = load_profiles();
    let profile = profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("配置集不存在: {}", id))?;

    serde_json::to_string_pretty(profile).map_err(|e| format!("导出配置集失败: {}", e))
}

/// 导入配置集（从 JSON 字符串）
#[tauri::command]
pub fn import_profile(json_data: String) -> Result<ConfigProfile, String> {
    let mut profile: ConfigProfile =
        serde_json::from_str(&json_data).map_err(|e| format!("解析配置集数据失败: {}", e))?;

    // 生成新的 ID 和时间戳以避免冲突
    profile.id = uuid::Uuid::new_v4().to_string();
    profile.active = false;
    let now = now_string();
    profile.created_at = now.clone();
    profile.updated_at = now;

    let mut profiles = load_profiles();
    profiles.push(profile.clone());
    save_profiles(&profiles)?;

    Ok(profile)
}

/// 切换配置集的启用/禁用状态
/// 启用时：备份 → 写入配置文件 → source 生效
/// 禁用时：仅标记为非活跃（不自动还原，用户可手动恢复备份）
#[tauri::command]
pub fn toggle_profile(id: String, active: bool) -> Result<Vec<String>, String> {
    let mut profiles = load_profiles();
    let profile = profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("配置集不存在: {}", id))?;

    profile.active = active;
    profile.updated_at = now_string();

    let mut modified_paths: Vec<String> = Vec::new();

    if active {
        // 启用：写入配置文件并生效
        for entry in &profile.entries {
            create_simple_backup(&entry.file_path)?;

            let target = PathBuf::from(&entry.file_path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败 {}: {}", parent.display(), e))?;
            }

            fs::write(&target, &entry.content)
                .map_err(|e| format!("写入文件失败 {}: {}", entry.file_path, e))?;

            modified_paths.push(entry.file_path.clone());
        }

        // 设置环境变量
        for var in &profile.env_vars {
            platform::set_env_variable(&var.name, &var.value, "user")
                .map_err(|e| format!("设置环境变量失败 {}={}: {}", var.name, var.value, e))?;
        }

        // 生效配置
        for path in &modified_paths {
            let _ = platform::source_config_file(path);
        }
    }

    save_profiles(&profiles)?;
    Ok(modified_paths)
}
