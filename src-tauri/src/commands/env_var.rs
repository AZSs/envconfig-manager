use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

use crate::utils::platform;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
    pub scope: String,
    #[serde(rename = "isCustom")]
    pub is_custom: bool,
    pub source: Option<String>,
}

/// 通过交互式 shell 获取完整环境变量（解决 .app 启动时环境缺失的问题）
fn get_full_env_vars() -> HashMap<String, String> {
    if cfg!(unix) {
        let home = dirs::home_dir().unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let is_zsh = shell.contains("zsh");

        // 构建一个脚本：先 source 常见的环境初始化文件，再输出 env
        // 某些文件（如 .cargo/env）在非交互模式下可能被跳过，需要显式 source
        let mut source_cmds = Vec::new();

        let extra_files = [
            ".cargo/env",
            ".nvm/nvm.sh",
            ".sdkman/bin/sdkman-init.sh",
        ];
        for f in &extra_files {
            let p = home.join(f);
            if p.exists() {
                source_cmds.push(format!("source '{}'", p.display()));
            }
        }

        let extra_sources = if source_cmds.is_empty() {
            String::new()
        } else {
            format!("{} && ", source_cmds.join(" && "))
        };

        // 使用 login shell 加载 profile + rc，再显式加载可能被跳过的文件
        let cmd = format!("{}env", extra_sources);
        let shell_bin = if is_zsh { "zsh" } else { "bash" };

        if let Ok(output) = Command::new(shell_bin)
            .args(["-ilc", &cmd])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut vars = HashMap::new();
                for line in stdout.lines() {
                    if let Some(pos) = line.find('=') {
                        let key = &line[..pos];
                        let value = &line[pos + 1..];
                        if !key.is_empty() && !key.contains(' ') {
                            vars.insert(key.to_string(), value.to_string());
                        }
                    }
                }
                // 同时合并进程自身的环境变量（某些是 macOS 注入的）
                for (k, v) in std::env::vars() {
                    vars.entry(k).or_insert(v);
                }
                if !vars.is_empty() {
                    return vars;
                }
            }
        }
    }
    // fallback: 使用进程自身环境
    std::env::vars().collect()
}

/// 获取所有环境变量
#[tauri::command]
pub fn get_env_variables() -> Result<Vec<EnvVariable>, String> {
    let vars = get_full_env_vars();
    let system_vars = platform::get_system_env_var_names();

    let mut result: Vec<EnvVariable> = vars
        .into_iter()
        .map(|(name, value)| {
            let is_system = system_vars.contains(&name.to_uppercase());
            EnvVariable {
                name: name.clone(),
                value,
                scope: if is_system {
                    "system".to_string()
                } else {
                    "user".to_string()
                },
                is_custom: !is_system,
                source: None,
            }
        })
        .collect();

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(result)
}

/// 设置环境变量
#[tauri::command]
pub fn set_env_variable(name: String, value: String, scope: String) -> Result<(), String> {
    // 格式校验
    if name.is_empty() {
        return Err("变量名不能为空".to_string());
    }
    if name.contains(' ') || name.contains('=') {
        return Err("变量名包含非法字符".to_string());
    }

    platform::set_env_variable(&name, &value, &scope)
}

/// 删除环境变量
#[tauri::command]
pub fn delete_env_variable(name: String, scope: String) -> Result<(), String> {
    platform::delete_env_variable(&name, &scope)
}

/// 搜索环境变量
#[tauri::command]
pub fn search_env_variables(keyword: String) -> Result<Vec<EnvVariable>, String> {
    let all = get_env_variables()?;
    let keyword_lower = keyword.to_lowercase();
    let filtered = all
        .into_iter()
        .filter(|v| {
            v.name.to_lowercase().contains(&keyword_lower)
                || v.value.to_lowercase().contains(&keyword_lower)
        })
        .collect();
    Ok(filtered)
}
