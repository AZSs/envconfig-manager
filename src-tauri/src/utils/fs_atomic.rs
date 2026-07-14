//! 原子文件写入工具。
//!
//! 元数据(`backups.json` / `profiles.json`)与配置文件(`.zshrc` 等)必须
//! 原子写,避免写入中途崩溃导致半截文件。策略:写同目录临时文件 → rename。

use std::path::Path;
use std::fs;

use serde::Serialize;

/// 将序列化结果原子写入 `path`。
pub fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("序列化失败: {}", e))?;
    write_bytes_atomic(path, &bytes)
}

/// 将纯文本原子写入 `path`,用于非 JSON 文件(如 `.zshrc`)。
pub fn write_text_atomic(path: &Path, content: &str) -> Result<(), String> {
    write_bytes_atomic(path, content.as_bytes())
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 临时文件与目标同目录,保证 rename 是原子的(同文件系统)。
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|e| format!("写入临时文件失败: {}", e))?;
    fs::rename(&tmp, path).map_err(|e| format!("替换文件失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Sample {
        name: String,
        n: u32,
    }

    #[test]
    fn writes_valid_json_that_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let target: PathBuf = dir.path().join("data.json");
        let value = Sample { name: "alpha".into(), n: 7 };

        write_json_atomic(&target, &value).unwrap();

        let read: Sample = serde_json::from_str(&fs::read_to_string(&target).unwrap()).unwrap();
        assert_eq!(read, value);
    }

    #[test]
    fn replaces_existing_content() {
        let dir = tempfile::tempdir().unwrap();
        let target: PathBuf = dir.path().join("data.json");
        fs::write(&target, "OLD CONTENT").unwrap();

        let value = Sample { name: "beta".into(), n: 1 };
        write_json_atomic(&target, &value).unwrap();

        let read: Sample = serde_json::from_str(&fs::read_to_string(&target).unwrap()).unwrap();
        assert_eq!(read, value);
    }

    #[test]
    fn does_not_leave_temp_file_behind() {
        let dir = tempfile::tempdir().unwrap();
        let target: PathBuf = dir.path().join("data.json");
        let value = Sample { name: "gamma".into(), n: 2 };

        write_json_atomic(&target, &value).unwrap();

        assert!(target.exists());
        let leftovers = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".tmp")
            })
            .count();
        assert_eq!(leftovers, 0, "发现残留的 .tmp 临时文件");
    }

    // ---------- write_text_atomic ----------

    #[test]
    fn write_text_writes_content() {
        let dir = tempfile::tempdir().unwrap();
        let target: PathBuf = dir.path().join(".zshrc");
        write_text_atomic(&target, "export FOO=bar\n").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "export FOO=bar\n");
    }

    #[test]
    fn write_text_replaces_existing() {
        let dir = tempfile::tempdir().unwrap();
        let target: PathBuf = dir.path().join(".zshrc");
        fs::write(&target, "OLD").unwrap();
        write_text_atomic(&target, "NEW CONTENT").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW CONTENT");
    }
}
