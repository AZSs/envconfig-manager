use std::collections::HashSet;
use std::process::Command;

#[cfg(unix)]
extern crate libc;

/// 检查当前进程是否拥有管理员/root 权限
pub fn is_admin() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args(["-Command", "[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)"])
            .output();
        matches!(output, Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "True")
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// 请求管理员权限执行命令（跨平台）
/// macOS: osascript 弹窗鉴权
/// Linux: pkexec (PolicyKit)
/// Windows: powershell Start-Process -Verb RunAs
pub fn run_with_elevation(command: &str, args: &[&str]) -> Result<String, String> {
    if cfg!(target_os = "macos") {
        // macOS: 使用 osascript 调起授权对话框
        let script = format!(
            "do shell script \"{} {}\" with administrator privileges",
            command,
            args.iter()
                .map(|a| a.replace('\\', "\\\\").replace('"', "\\\""))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("权限提升失败: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("User canceled") || stderr.contains("(-128)") {
                Err("用户取消了授权，本次操作已取消".to_string())
            } else {
                Err(format!("权限提升执行失败: {}", stderr))
            }
        }
    } else if cfg!(target_os = "linux") {
        // Linux: 使用 pkexec (PolicyKit)
        let output = Command::new("pkexec")
            .arg(command)
            .args(args)
            .output()
            .map_err(|e| format!("权限提升失败: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let code = output.status.code().unwrap_or(-1);
            if code == 126 {
                Err("用户取消了授权，本次操作已取消".to_string())
            } else {
                Err(format!("权限提升执行失败: {}", String::from_utf8_lossy(&output.stderr)))
            }
        }
    } else if cfg!(windows) {
        // Windows: 通过 powershell 以管理员身份运行
        let full_args = args.join(" ");
        let ps_cmd = format!(
            "Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs -Wait -PassThru | Select-Object -ExpandProperty ExitCode",
            command, full_args
        );
        let output = Command::new("powershell")
            .args(["-Command", &ps_cmd])
            .output()
            .map_err(|e| format!("权限提升失败: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!("权限提升执行失败: {}", String::from_utf8_lossy(&output.stderr)))
        }
    } else {
        Err("不支持的操作系统".to_string())
    }
}

/// 系统级操作前的权限校验：如果需要 system 级别且无权限，则提权
/// 返回 Ok(true) 表示已有权限或提权成功，Err 表示用户拒绝或失败
pub fn ensure_system_permission(scope: &str) -> Result<(), String> {
    if scope != "system" {
        return Ok(()); // 用户级操作无需提权
    }

    if cfg!(unix) {
        // Unix 下系统级环境变量写入用户配置文件，不需要 root
        // 但如果需要修改 /etc/environment 等系统文件，则需要提权
        // 当前实现写入用户 shell 配置，因此用户级即可
        Ok(())
    } else if cfg!(windows) {
        // Windows 修改 Machine 级环境变量需要管理员权限
        if !is_admin() {
            // 触发 UAC 提权确认
            Err("需要管理员权限才能修改系统环境变量。请在弹窗中确认授权。".to_string())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// 获取各平台的配置文件路径
pub fn get_config_file_paths() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut paths = Vec::new();

    if cfg!(unix) {
        let candidates = vec![
            home.join(".zshrc"),
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".profile"),
        ];
        for p in candidates {
            if p.exists() {
                paths.push(p.to_string_lossy().to_string());
            }
        }
    }

    if cfg!(windows) {
        // PowerShell profile path
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "echo $PROFILE"])
            .output()
        {
            let profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !profile.is_empty() {
                paths.push(profile);
            }
        }
    }

    paths
}

/// 根据文件名检测 shell 类型
pub fn detect_shell_type(filename: &str) -> String {
    match filename {
        name if name.contains("zsh") => "zsh".to_string(),
        name if name.contains("bash") => "bash".to_string(),
        name if name.contains("profile") && !name.contains("bash") => "bash".to_string(),
        name if name.contains("ps1") || name.contains("powershell") => "powershell".to_string(),
        _ => "bash".to_string(),
    }
}

/// 执行 source 使配置文件生效
pub fn source_config_file(path: &str) -> Result<String, String> {
    if cfg!(unix) {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let shell_name = if shell.contains("zsh") { "zsh" } else { "bash" };

        let output = Command::new(shell_name)
            .args(["-ilc", &format!("source '{}' && env", path)])
            .output()
            .map_err(|e| format!("执行 source 失败: {}", e))?;

        if output.status.success() {
            Ok("配置已生效".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("生效失败: {}", stderr))
        }
    } else if cfg!(windows) {
        let output = Command::new("powershell")
            .args(["-Command", &format!(". '{}'", path)])
            .output()
            .map_err(|e| format!("执行配置生效失败: {}", e))?;

        if output.status.success() {
            Ok("配置已生效".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("生效失败: {}", stderr))
        }
    } else {
        Err("不支持的操作系统".to_string())
    }
}

/// 获取常见系统级环境变量名称
pub fn get_system_env_var_names() -> HashSet<String> {
    let names = vec![
        "PATH", "HOME", "USER", "SHELL", "LANG", "LC_ALL", "TERM",
        "DISPLAY", "EDITOR", "VISUAL", "PAGER", "LOGNAME", "HOSTNAME",
        "PWD", "OLDPWD", "TMPDIR", "TEMP", "TMP",
        // Windows 常见
        "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "OS",
        "PROGRAMFILES", "PROGRAMDATA", "APPDATA", "LOCALAPPDATA",
        "USERPROFILE", "HOMEDRIVE", "HOMEPATH",
    ];
    names.into_iter().map(|s| s.to_string()).collect()
}

/// 设置环境变量（跨平台，含权限校验）
pub fn set_env_variable(name: &str, value: &str, scope: &str) -> Result<(), String> {
    // 系统级操作先校验权限
    ensure_system_permission(scope)?;

    if cfg!(unix) {
        let home = dirs::home_dir().unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_default();
        let config_path = if shell.contains("zsh") {
            home.join(".zshrc")
        } else {
            home.join(".bashrc")
        };

        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        let export_line = format!("export {}=\"{}\"", name, value);

        let pattern = format!("export {}=", name);
        let new_content = if content.contains(&pattern) {
            content
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with(&pattern) {
                        export_line.as_str()
                    } else {
                        line
                    }
                })
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            format!("{}\n{}", content.trim_end(), export_line)
        };

        std::fs::write(&config_path, new_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        std::env::set_var(name, value);
        Ok(())
    } else if cfg!(windows) {
        let scope_arg = if scope == "system" { "Machine" } else { "User" };

        if scope == "system" && !is_admin() {
            // 通过提权执行
            let cmd = format!(
                "[Environment]::SetEnvironmentVariable('{}', '{}', '{}')",
                name, value, scope_arg
            );
            run_with_elevation("powershell", &["-Command", &cmd])?;
            std::env::set_var(name, value);
            Ok(())
        } else {
            let cmd = format!(
                "[Environment]::SetEnvironmentVariable('{}', '{}', '{}')",
                name, value, scope_arg
            );
            let output = Command::new("powershell")
                .args(["-Command", &cmd])
                .output()
                .map_err(|e| format!("设置环境变量失败: {}", e))?;

            if output.status.success() {
                std::env::set_var(name, value);
                Ok(())
            } else {
                Err(format!(
                    "设置失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
    } else {
        Err("不支持的操作系统".to_string())
    }
}

/// 删除环境变量（跨平台，含权限校验）
pub fn delete_env_variable(name: &str, scope: &str) -> Result<(), String> {
    // 系统级操作先校验权限
    ensure_system_permission(scope)?;

    if cfg!(unix) {
        let home = dirs::home_dir().unwrap_or_default();
        let shell = std::env::var("SHELL").unwrap_or_default();
        let config_path = if shell.contains("zsh") {
            home.join(".zshrc")
        } else {
            home.join(".bashrc")
        };

        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        let pattern = format!("export {}=", name);
        let new_content: String = content
            .lines()
            .filter(|line| !line.trim_start().starts_with(&pattern))
            .collect::<Vec<&str>>()
            .join("\n");

        std::fs::write(&config_path, new_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        std::env::remove_var(name);
        Ok(())
    } else if cfg!(windows) {
        let scope_arg = if scope == "system" { "Machine" } else { "User" };

        if scope == "system" && !is_admin() {
            let cmd = format!(
                "[Environment]::SetEnvironmentVariable('{}', $null, '{}')",
                name, scope_arg
            );
            run_with_elevation("powershell", &["-Command", &cmd])?;
            std::env::remove_var(name);
            Ok(())
        } else {
            let cmd = format!(
                "[Environment]::SetEnvironmentVariable('{}', $null, '{}')",
                name, scope_arg
            );
            let output = Command::new("powershell")
                .args(["-Command", &cmd])
                .output()
                .map_err(|e| format!("删除环境变量失败: {}", e))?;

            if output.status.success() {
                std::env::remove_var(name);
                Ok(())
            } else {
                Err(format!(
                    "删除失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
    } else {
        Err("不支持的操作系统".to_string())
    }
}
