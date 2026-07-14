//! Shell 安全工具:变量值转义与变量名校验。
//!
//! 任何拼接到 shell 字符串的变量值,必须经 `shell_quote_double` 转义,
//! 避免写出坏语法或被注入。

/// 将变量值转为可安全嵌入双引号 shell 字符串的形式。
///
/// 双引号上下文中需转义的字符:`"` `\` `$` `` ` `` 与换行。
/// 返回值已包含外层双引号,例如 `hello` -> `"hello"`。
pub fn shell_quote_double(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' | '\\' | '$' | '`' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 校验是否为合法的 shell 变量名:`[A-Za-z_][A-Za-z0-9_]*`。
pub fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// 将路径转为可安全嵌入单引号 shell 字符串的形式。
///
/// 单引号内仅 `'` 需特殊处理,用 `'\''` 结束-转义-重启单引号。
/// 返回值已包含外层单引号。
pub fn shell_quote_single(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 2);
    out.push('\'');
    for c in path.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- shell_quote_double ----------

    #[test]
    fn quote_double_plain() {
        assert_eq!(shell_quote_double("hello"), "\"hello\"");
    }

    #[test]
    fn quote_double_empty() {
        assert_eq!(shell_quote_double(""), "\"\"");
    }

    #[test]
    fn quote_double_path_with_spaces() {
        assert_eq!(shell_quote_double("/usr/local/bin"), "\"/usr/local/bin\"");
    }

    #[test]
    fn quote_double_escapes_double_quote() {
        assert_eq!(shell_quote_double("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn quote_double_escapes_backslash() {
        assert_eq!(shell_quote_double("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn quote_double_escapes_dollar() {
        assert_eq!(shell_quote_double("a$b"), "\"a\\$b\"");
    }

    #[test]
    fn quote_double_escapes_backtick() {
        assert_eq!(shell_quote_double("a`b"), "\"a\\`b\"");
    }

    #[test]
    fn quote_double_escapes_newline() {
        assert_eq!(shell_quote_double("a\nb"), "\"a\\nb\"");
    }

    #[test]
    fn quote_double_value_with_expansion_is_neutralized() {
        // 危险值:尝试命令替换与变量展开,转义后应原样保留为字面量
        let dangerous = "$(rm -rf /)`whoami`$HOME";
        let quoted = shell_quote_double(dangerous);
        assert_eq!(quoted, "\"\\$(rm -rf /)\\`whoami\\`\\$HOME\"");
    }

    // ---------- is_valid_var_name ----------

    #[test]
    fn var_name_valid_simple() {
        assert!(is_valid_var_name("FOO"));
    }

    #[test]
    fn var_name_valid_underscore_prefix() {
        assert!(is_valid_var_name("_FOO"));
    }

    #[test]
    fn var_name_valid_with_digits_and_underscore() {
        assert!(is_valid_var_name("FOO_BAR_1"));
    }

    #[test]
    fn var_name_invalid_empty() {
        assert!(!is_valid_var_name(""));
    }

    #[test]
    fn var_name_invalid_leading_digit() {
        assert!(!is_valid_var_name("1FOO"));
    }

    #[test]
    fn var_name_invalid_dash() {
        assert!(!is_valid_var_name("FOO-BAR"));
    }

    #[test]
    fn var_name_invalid_space() {
        assert!(!is_valid_var_name("FOO BAR"));
    }

    #[test]
    fn var_name_invalid_equals() {
        assert!(!is_valid_var_name("FOO=BAR"));
    }

    // ---------- shell_quote_single ----------

    #[test]
    fn quote_single_plain() {
        assert_eq!(shell_quote_single("/Users/me/.zshrc"), "'/Users/me/.zshrc'");
    }

    #[test]
    fn quote_single_with_apostrophe() {
        // O'Reilly -> 'O'\''Reilly'
        assert_eq!(shell_quote_single("O'Reilly"), "'O'\\''Reilly'");
    }

    #[test]
    fn quote_single_empty() {
        assert_eq!(shell_quote_single(""), "''");
    }
}
