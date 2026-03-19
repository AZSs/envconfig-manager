use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[tauri::command]
pub async fn validate_config_file(
    path: String,
    content: String,
) -> Result<ValidationResult, String> {
    // 1. Syntax check
    let syntax_result = check_syntax(path.clone(), content.clone()).await?;
    let mut all_issues = syntax_result.issues;

    // 2. PATH existence check
    let path_issues = check_path_existence(content.clone()).await?;
    all_issues.extend(path_issues);

    // 3. Conflict detection
    let conflict_issues = detect_conflicts(&content);
    all_issues.extend(conflict_issues);

    // valid only when there are zero errors
    let valid = !all_issues.iter().any(|issue| issue.severity == "error");

    Ok(ValidationResult {
        file_path: path,
        valid,
        issues: all_issues,
    })
}

#[tauri::command]
pub async fn check_syntax(path: String, content: String) -> Result<ValidationResult, String> {
    let shell = if path.contains("zsh") {
        "zsh"
    } else {
        "bash"
    };

    let mut temp_file =
        tempfile::NamedTempFile::new().map_err(|e| format!("创建临时文件失败: {}", e))?;

    temp_file
        .write_all(content.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();

    let output = Command::new(shell)
        .arg("-n")
        .arg(&temp_path)
        .output()
        .map_err(|e| format!("执行语法检查失败: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut issues = Vec::new();

    // Bash format:  filename: line N: message
    // Zsh format:   filename:N: message
    let re_bash = Regex::new(r"^.+: line (\d+): (.+)$").unwrap();
    let re_zsh = Regex::new(r"^.+:(\d+): (.+)$").unwrap();

    for line in stderr.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let (line_num, message) = if let Some(caps) = re_bash.captures(line) {
            let n = caps[1].parse::<usize>().ok();
            let msg = caps[2].to_string();
            (n, msg)
        } else if let Some(caps) = re_zsh.captures(line) {
            let n = caps[1].parse::<usize>().ok();
            let msg = caps[2].to_string();
            (n, msg)
        } else {
            (None, line.to_string())
        };

        issues.push(ValidationIssue {
            severity: "error".to_string(),
            line: line_num,
            column: None,
            message,
            suggestion: None,
            category: "syntax".to_string(),
        });
    }

    let valid = issues.is_empty();

    Ok(ValidationResult {
        file_path: path,
        valid,
        issues,
    })
}

#[tauri::command]
pub async fn check_path_existence(content: String) -> Result<Vec<ValidationIssue>, String> {
    let re = Regex::new(r"(?m)export\s+(PATH|[\w_]*PATH[\w_]*)=(.*)")
        .map_err(|e| format!("正则表达式编译失败: {}", e))?;

    let home_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut issues = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        if let Some(caps) = re.captures(line) {
            let var_name = &caps[1];
            let raw_value = caps[2].to_string();

            // Remove surrounding quotes
            let value = raw_value
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .trim_start_matches('\'')
                .trim_end_matches('\'');

            for segment in value.split(':') {
                let segment = segment.trim();
                if segment.is_empty() {
                    continue;
                }

                // Skip segments that contain unexpanded variables (other than $HOME)
                let expanded = segment
                    .replace("$HOME", &home_dir)
                    .replace('~', &home_dir);

                // Skip if the segment still contains variable references like $FOO
                if expanded.contains('$') {
                    continue;
                }

                if !std::path::Path::new(&expanded).exists() {
                    issues.push(ValidationIssue {
                        severity: "warning".to_string(),
                        line: Some(line_idx + 1),
                        column: None,
                        message: format!(
                            "路径 '{}' 在 {} 中引用但不存在",
                            expanded, var_name
                        ),
                        suggestion: Some(format!(
                            "请确认路径 '{}' 是否正确，或创建该目录",
                            expanded
                        )),
                        category: "path".to_string(),
                    });
                }
            }
        }
    }

    Ok(issues)
}

fn detect_conflicts(content: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let re_export = Regex::new(r"(?m)^[ \t]*export\s+(\w+)=").unwrap();
    let re_alias = Regex::new(r"(?m)^[ \t]*alias\s+(\w+)=").unwrap();

    let mut export_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut alias_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        if let Some(caps) = re_export.captures(line) {
            let name = caps[1].to_string();
            export_map.entry(name).or_default().push(line_num);
        }

        if let Some(caps) = re_alias.captures(line) {
            let name = caps[1].to_string();
            alias_map.entry(name).or_default().push(line_num);
        }
    }

    for (name, lines) in &export_map {
        if lines.len() >= 2 {
            let lines_str = lines
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" 行和第 ");

            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                line: Some(lines[lines.len() - 1]),
                column: None,
                message: format!("变量 '{}' 被重复定义了 {} 次", name, lines.len()),
                suggestion: Some(format!(
                    "变量 '{}' 在第 {} 行重复定义，建议合并",
                    name, lines_str
                )),
                category: "conflict".to_string(),
            });
        }
    }

    for (name, lines) in &alias_map {
        if lines.len() >= 2 {
            let lines_str = lines
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" 行和第 ");

            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                line: Some(lines[lines.len() - 1]),
                column: None,
                message: format!("别名 '{}' 被重复定义了 {} 次", name, lines.len()),
                suggestion: Some(format!(
                    "别名 '{}' 在第 {} 行重复定义，建议合并",
                    name, lines_str
                )),
                category: "conflict".to_string(),
            });
        }
    }

    issues
}
