// 配置文件信息
export interface ConfigFile {
  path: string
  name: string
  lastModified: string
  content: string
  shellType: 'bash' | 'zsh' | 'powershell' | 'cmd'
}

// 环境变量
export interface EnvVariable {
  name: string
  value: string
  scope: 'user' | 'system'
  isCustom: boolean
  source?: string
}

// 备份快照
export interface BackupSnapshot {
  id: string
  fileName: string
  timestamp: string
  remark: string
  fileSize: number
  filePath: string
}

// 备份配置
export interface BackupConfig {
  maxSnapshots: number
  maxSizeMB: number
  backupDir: string
}

// === 配置校验 ===

export type ValidationSeverity = 'error' | 'warning' | 'info'

export interface ValidationIssue {
  severity: ValidationSeverity
  line: number | null
  column: number | null
  message: string
  suggestion: string | null
  category: 'syntax' | 'path' | 'conflict'
}

export interface ValidationResult {
  filePath: string
  valid: boolean
  issues: ValidationIssue[]
}

// === 配置集管理 ===

export interface ConfigProfileEntry {
  filePath: string
  content: string
}

export interface EnvVarEntry {
  name: string
  value: string
}

export interface ConfigProfile {
  id: string
  name: string
  description: string
  active: boolean
  createdAt: string
  updatedAt: string
  entries: ConfigProfileEntry[]
  envVars: EnvVarEntry[]
}

export interface ProfileDiffItem {
  filePath: string
  currentContent: string
  profileContent: string
}

export interface ProfileDiffResult {
  diffs: ProfileDiffItem[]
  hasUnsavedChanges: boolean
}

// === 主题设置 ===

export type ThemeMode = 'light' | 'dark'

export interface ThemeSettings {
  mode: ThemeMode
  editorFontSize: number
}
