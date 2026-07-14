# EnvConfig Manager 重构文档

> 版本:v1 草案 · 日期:2026-07-14
> 范围:以后端(`src-tauri/`)为核心,前端联动调整

---

## 1. 重构目标

当前项目是一个完成度约 70% 的 MVP:架构分层清晰、选型合理,但**关键写入路径的可靠性未补齐**,存在「把用户配置写坏」的真实风险,且部分功能「名实不符」。本次重构不推翻架构,而是:

1. **守住底线**:任何对用户 `.zshrc` / 环境变量的写入都必须「可回滚、不破坏语法、不丢元数据」。
2. **消除重复与割裂**:统一备份体系、统一配置写入入口、统一 shell 判定。
3. **如实承诺**:重新定义「生效」语义,文档与实现一致。
4. **补齐可验证性**:为核心写入/解析逻辑补充单元测试。

非目标:不改技术栈、不改 UI 框架、不引入数据库。

---

## 2. 现状评估

### 2.1 架构现状(值得保留)

```
前端 (Vue3 + Pinia)
  └─ invoke(command) ──► Tauri commands (src-tauri/src/commands/*.rs)
                            └─ utils/platform.rs  (跨平台抽象)
                            └─ 直接 fs / std::process
```

- 分层干净,命令命名规范,`lib.rs` 一处登记全部能力边界。
- 跨平台 `cfg!` 分支结构清楚。
- 错误统一 `Result<T, String>` 透传,中文化。

### 2.2 问题清单(按严重度)

| # | 严重度 | 模块 | 问题 | 后果 |
|---|--------|------|------|------|
| P1 | 🔴 致命 | `platform.rs:230` | `set_env_variable` 用 `format!("export {}=\"{}\"", name, value)` 写入,value 未做 shell 转义 | value 含 `"`/`$`/反引号/反斜杠时写出坏语法,用户下次 source 报错 |
| P2 | 🔴 致命 | `config.rs:65` | `write_config_file` 直接 `fs::write`,未触发自动备份 | P0 承诺的「修改前自动快照」落空,裸写覆盖用户配置 |
| P3 | 🔴 致命 | `backup.rs` vs `profile.rs` | 两套备份实现;`profile.rs::create_simple_backup` 不写 `backups.json` | profile 产生的备份在备份管理器里不可见、不可回滚 |
| P4 | 🟠 高 | `platform.rs:168` | `source_config_file` 实为子 shell `zsh -ilc "source ... && env"`,子进程退出后无残留 | 「一键生效」名实不符,对当前 app 与已开终端均无效 |
| P5 | 🟠 高 | `profile.rs:311` | `toggle_profile` 禁用时仅置 `active=false`,不还原;且其备份不可见(P3) | 用户「关掉」配置集后无法找回原配置 |
| P6 | 🟠 高 | `backup.rs:50` / `profile.rs:84` | 元数据 `fs::write` 非原子 | 写入中途崩溃 → `backups.json`/`profiles.json` 损坏,全量丢失 |
| P7 | 🟡 中 | `env_var.rs:18` | `get_full_env_vars` 每次起 login+interactive shell;`search_env_variables` 每次重新全量 | 搜索卡顿;`-i` 触发用户 rc 副作用 |
| P8 | 🟡 中 | `profile.rs:201` vs `:311` | `apply_profile` 与 `toggle_profile(active=true)` 写入逻辑几乎完全重复 | 维护双份,易漂移 |
| P9 | 🟡 中 | `validation.rs:53` vs `config.rs:42` | shell 判定各自实现(`path.contains("zsh")` vs `detect_shell_type`),不一致 | PowerShell/CMD 路径判定偏差 |
| P10 | 🟡 中 | `validation.rs:121` | PATH 路径用 `split(':')`,未处理 Windows `;` 分隔 | Windows 下 PATH 检测错误 |
| P11 | 🟡 中 | `validation.rs:69` | async 命令内用同步 `Command::output()` | 阻塞 tokio runtime 线程 |
| P12 | 🟡 中 | `platform.rs:37` | `run_with_elevation` 的 osascript 转义只处理 `\` 和 `"`,未处理 `'` | 含单引号的参数破坏 AppleScript |
| P13 | 🟢 低 | 全局 | 无任何单元测试 | 解析/写入逻辑无回归保护 |
| P14 | 🟢 低 | `backup.rs:103` | `auto_cleanup` 硬编码 `50`,未读 `BackupConfig` 且未按容量清理 | 容量策略(P0)未实现 |
| P15 | 🟢 低 | `env_var.rs:89` | 「系统级/用户级」靠硬编码名称集合判定,非真实来源 | 误判 scope |

---

## 3. 重构原则

1. **单一写入入口**:所有对用户配置文件的写入收敛到一个 `config_writer` 服务,内置「备份 → 原子写 → 记录」。
2. **单一备份服务**:备份创建、元数据登记、清理、回滚集中在 `backup` 服务,其它模块只调用,不自行复制文件。
3. **shell 安全**:任何拼接到 shell 字符串的变量值,必须经过转义函数。
4. **原子持久化**:JSON 元数据一律「写临时文件 → rename」。
5. **诚实语义**:无法做到的能力(注入已开终端)在文档与文案中如实降级表述。
6. **测试先行**:解析/转义/写入逻辑先有单测,再改实现。

---

## 4. 分模块重构方案

### 4.1 shell 转义与命令注入(P1, P12)

新增 `utils/shell.rs`,提供转义工具:

```rust
/// 将变量值转为可安全嵌入双引号 shell 字符串的形式
pub fn shell_quote_double(value: &str) -> String {
    // 双引号内需转义: $ ` " \ 换行
    let escaped: String = value.chars().flat_map(|c| match c {
        '"' => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        '$' => vec!['\\', '$'],
        '`' => vec!['\\', '`'],
        '\n' => vec!['\\', 'n'],
        c => vec![c],
    }).collect();
    format!("\"{}\"", escaped)
}

/// 变量名校验(仅允许 [A-Za-z_][A-Za-z0-9_]*)
pub fn is_valid_var_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}
```

改造点:
- `platform.rs::set_env_variable`:用 `shell_quote_double(&value)` 替换裸拼接;`name` 用 `is_valid_var_name` 校验,非法直接 `Err`。
- `platform.rs::source_config_file`:路径用单引号包裹并对值内的 `'` 做 `'\''` 转义(当前 `source '{}'` 对含 `'` 的路径不安全)。
- `platform.rs::run_with_elevation` (macOS 分支):补齐 `'` 转义,或改用 `do shell script ... with administrator privileges` 的列表参数形式避免字符串拼接。

### 4.2 备份体系统一化(P3, P14)

废除 `profile.rs::create_simple_backup`,所有备份走 `backup` 模块的统一函数(下沉为非 command 的内部函数,供其它模块调用):

```rust
// backup.rs
pub fn backup_file_internal(source_path: &str, remark: &str) -> Result<BackupSnapshot, String> {
    // 1. 复制文件到 ~/.envconfig/backups/{name}_{ts}.bak
    // 2. 写入 backups.json(原子写,P6)
    // 3. auto_cleanup 读取 BackupConfig(maxSnapshots + maxSizeMB)
}
```

`#[tauri::command] create_backup` 改为薄包装,调用 `backup_file_internal`。

清理策略补齐(P14):
- 同时按数量(`max_snapshots`)和容量(`max_size_mb`)清理,超出任一上限删最旧。
- 清理前可选通知前端(本期先静默清理并记日志,通知留后续)。

### 4.3 配置写入路径统一(P2, P6)

新增 `commands/config_writer` 逻辑(可放在 `config.rs` 内):

```rust
#[tauri::command]
pub fn write_config_file(path: String, content: String) -> Result<(), String> {
    // 1. 若文件存在 → backup_file_internal(path, "写入前自动备份")
    // 2. 原子写:写 {path}.tmp → rename 到 {path}
    // 3. (可选)记录操作日志
}
```

- `profile.rs::apply_profile` / `toggle_profile` 内的 `fs::write` 全部替换为调用同一写入函数(内部已含备份)。
- 元数据(`backups.json` / `profiles.json`)统一走 `utils/fs_atomic.rs::write_json_atomic`:

```rust
pub fn write_json_atomic(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?; // 原子替换
    Ok(())
}
```

### 4.4 「生效」语义重定义(P4)

当前 `source_config_file` 实际只是「语法可执行性校验」,无法影响已开终端或 app 自身进程。重构方案分两步:

**短期(本期)**:如实降级文案与返回值。
- 命令重命名/拆分:`validate_apply_config(path)` 返回结构化结果:
  - `syntax_ok: bool`
  - `message: "配置语法正确,新开终端将自动生效"`(明确「新开终端」)
- 前端按钮文案由「生效」改为「校验并应用」,提示语写清「对已打开的终端无效,需新开终端或手动 source」。
- README/feature.md 同步修订措辞。

**中期(后续里程碑)**:提供真实生效通道(择一):
- macOS/Linux:写入 `~/.envconfig/refresh-marker`,提供一个 shell 片段 `envconfig-refresh` 供用户在终端 `source` 一次完成刷新(降低手动 source 成本)。
- 或:app 内置终端(集成 xterm.js),在该终端内 source,保证至少 app 内生效。

> 本期只做「如实降级」,避免继续误导用户。

### 4.5 环境变量读取性能(P7)

- `get_full_env_vars` 的结果在前端 `envVar` store 缓存,提供 `refresh` 显式刷新;`search_env_variables` 改为纯前端内存过滤(删除该 command,或保留但标记 deprecated)。
- 去掉 `-i`(interactive),仅用 `-l`(login):避免触发 fortune/nvm 等交互式副作用;显式 source `.cargo/env` 等保留。
- 大 profile 场景:启动时异步预热一次,UI 不阻塞。

### 4.6 校验模块整合(P9, P10, P11)

- shell 判定统一走 `platform::detect_shell_type`(已存在),删除 `validation.rs` 内 `path.contains("zsh")` 的本地判定。
- `check_path_existence`:分隔符按平台选择(`:` on unix, `;` on windows)。
- async 命令内的同步 `Command::output()` 改用 `tokio::process::Command`,或整体用 `spawn_blocking` 包裹,避免阻塞 runtime。
- 校验结果增加 `shell_type` 字段,前端可据此切换 CodeMirror 语法。

### 4.7 配置集去重(P5, P8)

- 抽取 `apply_profile_entries(profile)`:备份 → 原子写 → set env,供 `apply_profile` 与 `toggle_profile(active=true)` 共用。
- `toggle_profile(active=false)`:本期明确产品决策——
  - 方案 A(推荐):禁用时**不自动还原**,但 UI 显著提示「已禁用,原配置快照见备份管理器(时间戳 XXX)」,并保证该快照已在 `backups.json` 中可见(依赖 4.2 统一备份)。
  - 方案 B:禁用时自动还原到启用前的备份。需要记录「启用前快照 id」到 profile,复杂度更高。
- 建议本期采用方案 A,并在 `ConfigProfile` 增加 `lastEnabledBackupId` 字段为方案 B 预留。

### 4.8 错误处理与类型(P15 及整体)

- 引入 `thiserror` 定义后端错误枚举,命令层 `From<E> -> Result<T, String>` 统一转中文文案;避免散落的 `format!`。
- scope 判定:Unix 下「系统级」改用「是否写入 `/etc`」的真实判定,而非名称集合;`get_system_env_var_names` 仅作展示分组的启发式提示,不作为写入路径依据。

---

## 5. 目标架构

```
前端 (Vue3 + Pinia)
  └─ invoke ──► commands/*.rs           (薄层,仅参数校验 + 调用 service)
                   ├─ services/config_writer.rs   ← 单一写入入口(备份+原子写)
                   ├─ services/backup.rs          ← 单一备份服务(创建/清理/回滚)
                   ├─ services/profile.rs         ← 调用 config_writer + backup
                   └─ utils/
                        ├─ shell.rs        (转义/校验)
                        ├─ fs_atomic.rs    (原子 JSON 写)
                        └─ platform.rs     (跨平台命令/权限)
```

关键变化:
- `commands/` 退化为薄层,业务逻辑下沉到 `services/`。
- 备份与写入成为可被任意模块复用的内部服务,不再各自 `fs::copy`。

---

## 6. 分阶段实施计划

### 里程碑 M1:写入安全底线(最高优先,建议先做)
- [ ] 新增 `utils/shell.rs` + 单测(转义、变量名校验)
- [ ] `set_env_variable` / `source_config_file` / `run_with_elevation` 接入转义
- [ ] 新增 `utils/fs_atomic.rs::write_json_atomic` + 单测
- [ ] `write_config_file` 改为「备份 + 原子写」
- [ ] 元数据写入全部走原子写
- **验收**:value 含特殊字符不再写坏 `.zshrc`;元数据写入中断可恢复。

### 里程碑 M2:备份体系统一
- [ ] `backup_file_internal` 抽取,`create_backup` 改薄包装
- [ ] 删除 `profile.rs::create_simple_backup`,profile 改调统一服务
- [ ] `auto_cleanup` 接入 `BackupConfig`(数量+容量)
- [ ] profile 备份在备份管理器可见、可回滚
- **验收**:任意途径产生的备份均出现在备份管理器,可一键回滚。

### 里程碑 M3:配置集去重与禁用语义
- [ ] 抽取 `apply_profile_entries`,`apply_profile`/`toggle_profile` 共用
- [ ] `toggle_profile(false)` 采用方案 A + UI 提示
- [ ] `ConfigProfile` 增加 `lastEnabledBackupId`(预留)
- **验收**:启用/禁用流程单一代码路径;禁用后用户能在备份管理器找回原配置。

### 里程碑 M4:生效语义如实化
- [ ] `source_config_file` 拆为「校验」+ 结构化返回
- [ ] 前端文案、README、feature.md 同步修订
- **验收**:文档与实现一致,不再宣称「无需重启终端」。

### 里程碑 M5:性能与校验整合
- [ ] 前端缓存环境变量,搜索改内存过滤
- [ ] `get_full_env_vars` 去 `-i`
- [ ] 校验 shell 判定统一、PATH 分隔符跨平台、async 不阻塞
- **验收**:搜索无卡顿;Windows PATH 检测正确。

### 里程碑 M6:测试与错误体系
- [ ] 引入 `thiserror` 错误枚举
- [ ] 补 `shell.rs` / `fs_atomic.rs` / config 写入 / backup 解析的单测
- [ ] scope 真实来源判定
- **验收**:核心写入/解析逻辑有回归保护;CI 跑通 `cargo test`。

---

## 7. 测试策略

| 层级 | 工具 | 覆盖重点 |
|------|------|----------|
| 单元(Rust) | `cargo test` | shell 转义、变量名校验、原子写、backup 元数据读写与清理、profile 解析 |
| 集成(Rust) | `cargo test` + 临时目录 | `write_config_file` 端到端(备份生成 + 内容正确 + 原子性) |
| 前端 | Vitest(待引入) | envVar store 缓存与搜索过滤逻辑 |
| 手工 | — | macOS zsh / bash、Windows PowerShell 各跑一次「改含特殊字符的变量 → 回滚」 |

优先补 M1 涉及模块的单测,其余随里程碑补齐。

---

## 8. 风险与回滚

| 风险 | 缓解 |
|------|------|
| 重构写入路径引入新 bug,反而写坏配置 | M1 先补单测再改实现;保留旧 `fs::write` 路径一个版本,通过开关回退 |
| 备份统一后,旧 `create_simple_backup` 产生的孤立 `.bak` 文件不在元数据中 | 提供一次性迁移:启动时扫描 `~/.envconfig/backups/*.bak`,补登 `backups.json` |
| 「生效」文案变更影响用户预期 | 在 release notes 与 UI 内显眼提示 |
| `thiserror` 引入改动面大 | 放 M6,与功能解耦,可独立合入 |

---

## 9. 建议的落地顺序

如果时间有限,**只做 M1 + M2** 即可消除「写坏用户配置」与「备份不可回滚」两个最致命问题,投入产出比最高。M3~M6 可在后续迭代逐步推进。

M1 可立即开始的第一个 PR:`utils/shell.rs` + 转义单测,并接入 `set_env_variable`。改动小、隔离、可独立验证。
