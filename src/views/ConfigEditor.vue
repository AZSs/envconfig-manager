<template>
  <div class="config-editor">
    <!-- Toast notification -->
    <Transition name="toast">
      <div v-if="toast.visible" class="toast" :class="toast.type">
        {{ toast.message }}
      </div>
    </Transition>

    <!-- Left: file list panel -->
    <aside class="file-panel">
      <div class="file-panel-header">
        <h3>Configuration Files</h3>
      </div>
      <div v-if="configStore.loading" class="file-list-loading">
        Loading...
      </div>
      <ul v-else class="file-list">
        <li
          v-for="file in configStore.files"
          :key="file.path"
          class="file-item"
          :class="{ active: configStore.currentFile?.path === file.path }"
          @click="selectFile(file.path)"
        >
          <div class="file-name">{{ file.name }}</div>
          <div class="file-path">{{ file.path }}</div>
          <div class="file-modified">{{ file.lastModified }}</div>
        </li>
      </ul>
    </aside>

    <!-- Right: editor area -->
    <main class="editor-area">
      <template v-if="configStore.currentFile">
        <div class="editor-toolbar">
          <span class="toolbar-filename">{{ configStore.currentFile.name }}</span>
          <div class="toolbar-actions">
            <span v-if="authRemaining" class="auth-badge" @click="revokeAuth" title="点击锁定，需重新确认权限">
              &#128275; 已授权 {{ authRemaining }}
            </span>
            <button class="btn btn-primary" @click="requestSave">保存</button>
            <button class="btn btn-success" @click="requestApply">生效</button>
            <button class="btn btn-ghost" @click="handleBackup">备份</button>
          </div>
        </div>
        <div class="editor-container" ref="editorRef"></div>
        <ValidationPanel
          :result="validationStore.lastResult"
          @goto-line="gotoEditorLine"
        />
      </template>
      <div v-else class="empty-state">
        <p>从左侧面板选择一个配置文件开始编辑</p>
      </div>
    </main>

    <!-- 权限确认弹窗 -->
    <div v-if="showConfirmModal" class="modal-overlay" @click.self="cancelConfirm">
      <div class="modal-card">
        <h3 class="modal-title">{{ confirmTitle }}</h3>
        <p class="modal-text">{{ confirmMessage }}</p>
        <div class="modal-info">
          <div class="info-row">
            <span class="info-label">目标文件</span>
            <span class="info-value mono">{{ configStore.currentFile?.path }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">操作类型</span>
            <span class="info-value">{{ confirmAction === 'save' ? '保存修改' : '生效配置 (source)' }}</span>
          </div>
        </div>
        <div class="modal-note">
          <span class="note-icon">&#9432;</span>
          系统将在操作前自动创建备份快照
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="cancelConfirm">取消</button>
          <button class="btn btn-primary" @click="executeConfirm">确认{{ confirmAction === 'save' ? '保存' : '生效' }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { useConfigStore } from '../stores/config'
import { useBackupStore } from '../stores/backup'
import { useValidationStore } from '../stores/validation'
import { useThemeStore } from '../stores/theme'
import ValidationPanel from '../components/ValidationPanel.vue'
import { EditorView, lineNumbers } from '@codemirror/view'
import { EditorState, Compartment } from '@codemirror/state'
import { javascript } from '@codemirror/lang-javascript'
import { oneDark } from '@codemirror/theme-one-dark'
import { linter, lintGutter } from '@codemirror/lint'

const configStore = useConfigStore()
const backupStore = useBackupStore()
const validationStore = useValidationStore()
const themeStore = useThemeStore()

const editorRef = ref<HTMLElement | null>(null)
let editorView: EditorView | null = null
const themeCompartment = new Compartment()
const fontSizeCompartment = new Compartment()

// Toast
const toast = ref<{ visible: boolean; message: string; type: 'success' | 'error' }>({
  visible: false,
  message: '',
  type: 'success',
})
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(message: string, type: 'success' | 'error' = 'success') {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = { visible: true, message, type }
  toastTimer = setTimeout(() => {
    toast.value.visible = false
  }, 3000)
}

// 确认弹窗 + 授权会话
const showConfirmModal = ref(false)
const confirmAction = ref<'save' | 'apply'>('save')
const confirmTitle = ref('')
const confirmMessage = ref('')

// 授权会话：确认一次后 1 小时内免确认
const AUTH_DURATION_MS = 60 * 60 * 1000 // 1 小时
let authGrantedAt: number | null = null
const authRemaining = ref('') // 剩余时间显示
let authTimer: ReturnType<typeof setInterval> | null = null

function isAuthorized(): boolean {
  if (!authGrantedAt) return false
  return Date.now() - authGrantedAt < AUTH_DURATION_MS
}

function grantAuth() {
  authGrantedAt = Date.now()
  startAuthTimer()
}

function revokeAuth() {
  authGrantedAt = null
  authRemaining.value = ''
  if (authTimer) {
    clearInterval(authTimer)
    authTimer = null
  }
}

function startAuthTimer() {
  if (authTimer) clearInterval(authTimer)
  updateAuthRemaining()
  authTimer = setInterval(() => {
    updateAuthRemaining()
    if (!isAuthorized()) {
      revokeAuth()
    }
  }, 1000)
}

function updateAuthRemaining() {
  if (!authGrantedAt) {
    authRemaining.value = ''
    return
  }
  const elapsed = Date.now() - authGrantedAt
  const remaining = Math.max(0, AUTH_DURATION_MS - elapsed)
  const mins = Math.floor(remaining / 60000)
  const secs = Math.floor((remaining % 60000) / 1000)
  authRemaining.value = `${mins}:${secs.toString().padStart(2, '0')}`
}

function requestSave() {
  if (!configStore.currentFile) return
  if (isAuthorized()) {
    doSave()
    return
  }
  confirmAction.value = 'save'
  confirmTitle.value = '确认保存配置文件'
  confirmMessage.value = `即将修改配置文件 ${configStore.currentFile.name}，此操作将覆盖原文件内容。`
  showConfirmModal.value = true
}

function requestApply() {
  if (!configStore.currentFile) return
  if (isAuthorized()) {
    doApply()
    return
  }
  confirmAction.value = 'apply'
  confirmTitle.value = '确认生效配置文件'
  confirmMessage.value = `即将执行 source ${configStore.currentFile.name}，使配置立即生效到当前环境。`
  showConfirmModal.value = true
}

function cancelConfirm() {
  showConfirmModal.value = false
}

async function executeConfirm() {
  showConfirmModal.value = false
  grantAuth()
  if (confirmAction.value === 'save') {
    await doSave()
  } else {
    await doApply()
  }
}

// Editor
function getEditorContent(): string {
  return editorView?.state.doc.toString() ?? ''
}

function gotoEditorLine(line: number) {
  if (!editorView) return
  const lineInfo = editorView.state.doc.line(Math.min(line, editorView.state.doc.lines))
  editorView.dispatch({
    selection: { anchor: lineInfo.from },
    scrollIntoView: true,
  })
  editorView.focus()
}

function editorThemeExtension() {
  return themeStore.mode === 'dark' ? oneDark : []
}

function editorFontSizeExtension() {
  return EditorView.theme({
    '.cm-content': { fontSize: `${themeStore.editorFontSize}px` },
    '.cm-gutters': { fontSize: `${themeStore.editorFontSize}px` },
  })
}

function createEditor(content: string) {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }
  if (!editorRef.value) return

  const state = EditorState.create({
    doc: content,
    extensions: [
      lineNumbers(),
      javascript(),
      themeCompartment.of(editorThemeExtension()),
      fontSizeCompartment.of(editorFontSizeExtension()),
      lintGutter(),
      linter(() => {
        const result = validationStore.lastResult
        if (!result || !result.issues.length) return []
        return result.issues
          .filter(issue => issue.line != null)
          .map(issue => {
            const line = editorView!.state.doc.line(Math.min(issue.line!, editorView!.state.doc.lines))
            return {
              from: line.from,
              to: line.to,
              severity: issue.severity === 'error' ? 'error' as const : 'warning' as const,
              message: issue.message,
            }
          })
      }),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && configStore.currentFile) {
          configStore.setDirty(true)
          validationStore.debouncedSyntaxCheck(
            configStore.currentFile.path,
            update.state.doc.toString()
          )
        }
      }),
      EditorView.theme({
        '&': { height: '100%' },
        '.cm-scroller': { overflow: 'auto' },
      }),
    ],
  })

  editorView = new EditorView({
    state,
    parent: editorRef.value,
  })
}

// React to theme / font size changes
watch(() => themeStore.mode, () => {
  if (editorView) {
    editorView.dispatch({
      effects: themeCompartment.reconfigure(editorThemeExtension()),
    })
  }
})

watch(() => themeStore.editorFontSize, () => {
  if (editorView) {
    editorView.dispatch({
      effects: fontSizeCompartment.reconfigure(editorFontSizeExtension()),
    })
  }
})

async function selectFile(path: string) {
  await configStore.readFile(path)
  await nextTick()
  if (configStore.currentFile) {
    createEditor(configStore.currentFile.content ?? '')
    validationStore.clearValidation()
  }
}

/** 保存：自动备份 → 写入文件 */
async function doSave() {
  if (!configStore.currentFile) return
  try {
    const content = getEditorContent()
    const validation = await validationStore.validateFile(configStore.currentFile.path, content)
    if (!validation.valid) {
      showToast('存在语法错误，请修复后再保存', 'error')
      return
    }
    // 自动备份
    await backupStore.createBackup(configStore.currentFile.path, '保存前自动备份')
    // 写入文件
    await configStore.saveFile(configStore.currentFile.path, getEditorContent())
    configStore.setDirty(false)
    showToast('文件已保存（已自动创建备份）', 'success')
  } catch {
    showToast('保存失败', 'error')
  }
}

/** 生效：自动备份 → source 配置文件 */
async function doApply() {
  if (!configStore.currentFile) return
  try {
    // 自动备份
    await backupStore.createBackup(configStore.currentFile.path, '生效前自动备份')
    // 执行 source
    await configStore.applyFile(configStore.currentFile.path)
    showToast('配置已生效（已自动创建备份）', 'success')
  } catch {
    showToast('生效失败', 'error')
  }
}

async function handleBackup() {
  if (!configStore.currentFile) return
  try {
    await backupStore.createBackup(configStore.currentFile.path, '手动备份')
    showToast('备份创建成功', 'success')
  } catch {
    showToast('备份创建失败', 'error')
  }
}

onMounted(async () => {
  await configStore.scanFiles()
  // 自动选中第一个文件并加载内容
  if (configStore.files.length > 0) {
    await selectFile(configStore.files[0].path)
  }
})

onBeforeUnmount(() => {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }
  if (toastTimer) clearTimeout(toastTimer)
  if (authTimer) clearInterval(authTimer)
})
</script>

<style scoped>
.config-editor {
  display: flex;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
  position: relative;
}

/* Toast */
.toast {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 24px;
  border-radius: 6px;
  font-size: 14px;
  z-index: 100;
  pointer-events: none;
}
.toast.success {
  background: var(--success);
  color: var(--bg-primary);
}
.toast.error {
  background: var(--danger);
  color: var(--bg-primary);
}
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.3s, transform 0.3s;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-12px);
}

/* File panel */
.file-panel {
  width: 260px;
  min-width: 260px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.file-panel-header {
  padding: 16px;
  border-bottom: 1px solid var(--border);
}
.file-panel-header h3 {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 600;
}
.file-list-loading {
  padding: 16px;
  color: var(--text-muted);
  font-size: 13px;
}
.file-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}
.file-item {
  padding: 10px 16px;
  cursor: pointer;
  border-bottom: 1px solid var(--border);
  transition: background 0.15s;
}
.file-item:hover {
  background: var(--bg-hover);
}
.file-item.active {
  background: var(--bg-surface);
  border-left: 3px solid var(--accent);
  padding-left: 13px;
}
.file-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}
.file-path {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  word-break: break-all;
}
.file-modified {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* Editor area */
.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}
.toolbar-filename {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}
.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.auth-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 11px;
  background: rgba(166, 227, 161, 0.12);
  color: var(--success);
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
  white-space: nowrap;
}
.auth-badge:hover {
  background: rgba(166, 227, 161, 0.25);
}
.editor-container {
  flex: 1;
  overflow: hidden;
}
.editor-container :deep(.cm-editor) {
  height: 100%;
}

/* Empty state */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.empty-state p {
  color: var(--text-muted);
  font-size: 15px;
}

/* 确认弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.modal-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
  width: 480px;
  max-width: 90vw;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.modal-text {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 16px;
}

.modal-info {
  background: var(--bg-surface);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 12px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.info-label {
  font-size: 12px;
  color: var(--text-muted);
  flex: 0 0 64px;
}

.info-value {
  font-size: 13px;
  color: var(--text-primary);
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.modal-note {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--accent);
  background: rgba(137, 180, 250, 0.08);
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 16px;
}

.note-icon {
  font-size: 14px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
