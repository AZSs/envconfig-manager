# EnvConfig Manager

跨平台桌面配置文件与环境变量管理工具，基于 Tauri 2 + Vue 3 + TypeScript 构建。

## 功能特性

### P0 - 核心功能
- **配置文件管理** — 扫描并编辑 `.zshrc`、`.bashrc`、`.bash_profile` 等配置文件，内置 CodeMirror 6 代码编辑器
- **环境变量管理** — 查看/新增/编辑/删除环境变量，支持按分组展示和搜索过滤
- **备份回滚** — 操作前自动创建备份快照，支持手动备份与一键还原
- **权限提升** — macOS/Linux 通过系统授权对话框提权，Windows 通过 UAC

### P1 - 增强功能
- **配置校验** — 实时语法检查（`bash -n`/`zsh -n`）、PATH 路径存在性检测、重复 export/alias 冲突检测
- **多配置切换** — SwitchHosts 风格的配置集管理，toggle 开关快速启用/禁用，支持导入/导出

### P3 - 个性化
- **主题设置** — 浅色/深色主题切换（Catppuccin 配色），编辑器字体大小调整
- **自动更新** — 基于 GitHub Releases 的应用内自动更新

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x (Rust) |
| 前端 | Vue 3 + TypeScript + Vite |
| 状态管理 | Pinia |
| 路由 | Vue Router 4 |
| 编辑器 | CodeMirror 6 |
| 样式 | CSS Variables (Catppuccin 主题) |

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri:dev

# 构建 DMG
npm run tauri:build
```

## 系统要求

- macOS 12+ (Apple Silicon / Intel)
- Node.js 18+
- Rust 1.70+

## License

MIT
