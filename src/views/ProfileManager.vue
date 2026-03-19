<template>
  <div class="profile-manager">
    <!-- Toast notification -->
    <Transition name="toast">
      <div v-if="toast.visible" class="toast" :class="toast.type">
        {{ toast.message }}
      </div>
    </Transition>

    <!-- Left: profile list panel (SwitchHosts style) -->
    <aside class="profile-panel">
      <div class="profile-panel-header">
        <h3>配置集</h3>
        <div class="header-actions">
          <button class="btn btn-ghost btn-sm" @click="openImportModal" title="导入">&#8615;</button>
          <button class="btn btn-primary btn-sm" @click="openCreateModal">+ 新建</button>
        </div>
      </div>
      <div v-if="profileStore.loading" class="profile-list-loading">
        加载中...
      </div>
      <ul v-else class="profile-list">
        <li
          v-for="profile in profileStore.profiles"
          :key="profile.id"
          class="profile-item"
          :class="{ active: selectedProfile?.id === profile.id }"
          @click="selectProfile(profile)"
        >
          <div class="profile-item-main">
            <div class="profile-icon" :class="{ on: profile.active }">
              {{ profile.active ? '&#9679;' : '&#9675;' }}
            </div>
            <div class="profile-info">
              <div class="profile-name">{{ profile.name }}</div>
              <div class="profile-desc">{{ profile.description || '无描述' }}</div>
            </div>
          </div>
          <label class="toggle-switch" @click.stop>
            <input
              type="checkbox"
              :checked="profile.active"
              @change="handleToggle(profile, ($event.target as HTMLInputElement).checked)"
            />
            <span class="toggle-slider"></span>
          </label>
        </li>
      </ul>
    </aside>

    <!-- Right: profile detail / editor -->
    <main class="detail-area">
      <template v-if="selectedProfile">
        <!-- Header -->
        <div class="detail-toolbar">
          <div class="detail-toolbar-left">
            <span class="detail-status" :class="{ on: selectedProfile.active }">
              {{ selectedProfile.active ? '已启用' : '未启用' }}
            </span>
            <input
              v-if="editing"
              v-model="editName"
              class="edit-name-input"
              placeholder="配置集名称"
            />
            <span v-else class="detail-title">{{ selectedProfile.name }}</span>
          </div>
          <div class="toolbar-actions">
            <template v-if="editing">
              <button class="btn btn-primary" @click="handleSave">保存</button>
              <button class="btn btn-ghost" @click="cancelEdit">取消</button>
            </template>
            <template v-else>
              <button class="btn btn-ghost" @click="startEdit">编辑</button>
              <button class="btn btn-ghost" @click="handleExport">导出</button>
              <button class="btn btn-danger" @click="openDeleteModal">删除</button>
            </template>
          </div>
        </div>

        <!-- Content area -->
        <div class="detail-content">
          <template v-if="editing">
            <!-- Edit mode -->
            <div class="form-group">
              <label>描述</label>
              <input v-model="editDescription" type="text" placeholder="配置集描述（可选）" />
            </div>

            <!-- Config Entries -->
            <div class="section">
              <div class="section-header">
                <h4>配置文件</h4>
                <button class="btn btn-ghost btn-sm" @click="addEntry">+ 添加</button>
              </div>
              <div v-for="(entry, idx) in editEntries" :key="idx" class="entry-card">
                <div class="entry-header">
                  <select v-model="entry.filePath" class="entry-select">
                    <option value="" disabled>选择配置文件</option>
                    <option v-for="file in configStore.files" :key="file.path" :value="file.path">
                      {{ file.name }} ({{ file.path }})
                    </option>
                  </select>
                  <button class="btn btn-danger btn-sm" @click="removeEntry(idx)">移除</button>
                </div>
                <textarea v-model="entry.content" class="entry-content" rows="8" placeholder="配置内容..." />
              </div>
            </div>

            <!-- Environment Variables -->
            <div class="section">
              <div class="section-header">
                <h4>环境变量</h4>
                <button class="btn btn-ghost btn-sm" @click="addEnvVar">+ 添加</button>
              </div>
              <table v-if="editEnvVars.length > 0" class="env-table">
                <thead>
                  <tr><th>Name</th><th>Value</th><th></th></tr>
                </thead>
                <tbody>
                  <tr v-for="(envVar, idx) in editEnvVars" :key="idx">
                    <td><input v-model="envVar.name" type="text" placeholder="变量名" class="env-input" /></td>
                    <td><input v-model="envVar.value" type="text" placeholder="变量值" class="env-input" /></td>
                    <td class="env-action-cell"><button class="btn btn-danger btn-sm" @click="removeEnvVar(idx)">×</button></td>
                  </tr>
                </tbody>
              </table>
            </div>
          </template>

          <template v-else>
            <!-- Read-only view -->
            <div v-if="selectedProfile.description" class="readonly-desc">
              {{ selectedProfile.description }}
            </div>

            <div v-if="selectedProfile.entries.length > 0" class="section">
              <div class="section-header"><h4>配置文件 ({{ selectedProfile.entries.length }})</h4></div>
              <div v-for="(entry, idx) in selectedProfile.entries" :key="idx" class="readonly-entry">
                <div class="readonly-entry-path">{{ shortPath(entry.filePath) }}</div>
                <pre class="readonly-entry-content">{{ entry.content }}</pre>
              </div>
            </div>

            <div v-if="selectedProfile.envVars.length > 0" class="section">
              <div class="section-header"><h4>环境变量 ({{ selectedProfile.envVars.length }})</h4></div>
              <div class="readonly-env-list">
                <div v-for="(v, idx) in selectedProfile.envVars" :key="idx" class="readonly-env-row">
                  <span class="env-name">{{ v.name }}</span>
                  <span class="env-eq">=</span>
                  <span class="env-val">{{ v.value }}</span>
                </div>
              </div>
            </div>

            <div v-if="selectedProfile.entries.length === 0 && selectedProfile.envVars.length === 0" class="empty-hint">
              此配置集为空，点击「编辑」添加配置内容
            </div>
          </template>
        </div>
      </template>

      <!-- Empty state -->
      <div v-else class="empty-state">
        <p>从左侧选择或新建配置集</p>
      </div>
    </main>

    <!-- Create Profile Modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal-card">
        <h3>新建配置集</h3>
        <div class="form-group">
          <label>名称</label>
          <input v-model="createForm.name" type="text" placeholder="请输入配置集名称" />
        </div>
        <div class="form-group">
          <label>描述</label>
          <input v-model="createForm.description" type="text" placeholder="可选：配置集描述" />
        </div>
        <div class="form-group checkbox-group">
          <label class="checkbox-label">
            <input v-model="createForm.captureCurrentConfig" type="checkbox" />
            <span>从当前配置创建（快照当前配置文件内容）</span>
          </label>
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showCreateModal = false">取消</button>
          <button class="btn btn-primary" @click="handleCreate">创建</button>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteModal" class="modal-overlay" @click.self="showDeleteModal = false">
      <div class="modal-card">
        <h3>删除确认</h3>
        <p>确认删除配置集「{{ selectedProfile?.name }}」？</p>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showDeleteModal = false">取消</button>
          <button class="btn btn-danger" @click="handleDelete">确认删除</button>
        </div>
      </div>
    </div>

    <!-- Import Modal -->
    <div v-if="showImportModal" class="modal-overlay" @click.self="showImportModal = false">
      <div class="modal-card">
        <h3>导入配置集</h3>
        <div class="form-group">
          <label>粘贴 JSON 数据</label>
          <textarea v-model="importJson" rows="10" placeholder="请粘贴配置集 JSON 数据..." class="import-textarea" />
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showImportModal = false">取消</button>
          <button class="btn btn-primary" @click="handleImport">导入</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useProfileStore } from '../stores/profile'
import { useConfigStore } from '../stores/config'
import type { ConfigProfile, ConfigProfileEntry, EnvVarEntry } from '../types'

const profileStore = useProfileStore()
const configStore = useConfigStore()

// Selected profile
const selectedProfile = ref<ConfigProfile | null>(null)

// Edit mode
const editing = ref(false)
const editName = ref('')
const editDescription = ref('')
const editEntries = ref<ConfigProfileEntry[]>([])
const editEnvVars = ref<EnvVarEntry[]>([])

// Modals
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const showImportModal = ref(false)

// Create form
const createForm = ref({
  name: '',
  description: '',
  captureCurrentConfig: false,
})

// Import
const importJson = ref('')

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
  toastTimer = setTimeout(() => { toast.value.visible = false }, 3000)
}

function shortPath(filePath: string): string {
  return filePath.split('/').pop() ?? filePath
}

// Select profile (view mode)
function selectProfile(profile: ConfigProfile) {
  selectedProfile.value = profile
  editing.value = false
}

// Edit mode
function startEdit() {
  if (!selectedProfile.value) return
  editName.value = selectedProfile.value.name
  editDescription.value = selectedProfile.value.description
  editEntries.value = selectedProfile.value.entries.map(e => ({ ...e }))
  editEnvVars.value = selectedProfile.value.envVars.map(v => ({ ...v }))
  editing.value = true
}

function cancelEdit() {
  editing.value = false
}

// Entry management
function addEntry() { editEntries.value.push({ filePath: '', content: '' }) }
function removeEntry(idx: number) { editEntries.value.splice(idx, 1) }
function addEnvVar() { editEnvVars.value.push({ name: '', value: '' }) }
function removeEnvVar(idx: number) { editEnvVars.value.splice(idx, 1) }

// Toggle (SwitchHosts style)
async function handleToggle(profile: ConfigProfile, active: boolean) {
  try {
    await profileStore.toggleProfile(profile.id, active)
    // Refresh selectedProfile if it's the toggled one
    if (selectedProfile.value?.id === profile.id) {
      const updated = profileStore.profiles.find(p => p.id === profile.id)
      if (updated) selectedProfile.value = updated
    }
    showToast(active ? `已启用「${profile.name}」` : `已禁用「${profile.name}」`, 'success')
  } catch {
    showToast(active ? '启用失败' : '禁用失败', 'error')
  }
}

// Save
async function handleSave() {
  if (!selectedProfile.value) return
  try {
    await profileStore.updateProfile(
      selectedProfile.value.id,
      editName.value,
      editDescription.value,
      editEntries.value,
      editEnvVars.value,
    )
    const updated = profileStore.profiles.find(p => p.id === selectedProfile.value!.id)
    if (updated) selectedProfile.value = updated
    editing.value = false
    showToast('配置集已保存', 'success')
  } catch {
    showToast('保存失败', 'error')
  }
}

// Export
async function handleExport() {
  if (!selectedProfile.value) return
  try {
    const jsonStr = await profileStore.exportProfile(selectedProfile.value.id)
    const blob = new Blob([jsonStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${selectedProfile.value.name}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    showToast('导出成功', 'success')
  } catch {
    showToast('导出失败', 'error')
  }
}

// Delete
function openDeleteModal() { showDeleteModal.value = true }

async function handleDelete() {
  if (!selectedProfile.value) return
  try {
    await profileStore.deleteProfile(selectedProfile.value.id)
    selectedProfile.value = null
    showDeleteModal.value = false
    showToast('配置集已删除', 'success')
  } catch {
    showToast('删除失败', 'error')
  }
}

// Create
function openCreateModal() {
  createForm.value = { name: '', description: '', captureCurrentConfig: false }
  showCreateModal.value = true
}

async function handleCreate() {
  if (!createForm.value.name) return
  try {
    let entries: ConfigProfileEntry[] = []
    const envVars: EnvVarEntry[] = []

    if (createForm.value.captureCurrentConfig) {
      for (const file of configStore.files) {
        await configStore.readFile(file.path)
        const loaded = configStore.currentFile
        if (loaded && loaded.path === file.path) {
          entries.push({ filePath: file.path, content: loaded.content ?? '' })
        }
      }
    }

    const profile = await profileStore.createProfile(
      createForm.value.name,
      createForm.value.description,
      entries,
      envVars,
    )
    showCreateModal.value = false
    if (profile) selectProfile(profile)
    showToast('配置集已创建', 'success')
  } catch {
    showToast('创建失败', 'error')
  }
}

// Import
function openImportModal() {
  importJson.value = ''
  showImportModal.value = true
}

async function handleImport() {
  if (!importJson.value.trim()) return
  try {
    const profile = await profileStore.importProfile(importJson.value)
    showImportModal.value = false
    if (profile) selectProfile(profile)
    showToast('导入成功', 'success')
  } catch {
    showToast('导入失败', 'error')
  }
}

onMounted(() => {
  profileStore.loadProfiles()
  configStore.scanFiles()
})
</script>

<style scoped>
.profile-manager {
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
.toast.success { background: var(--success); color: var(--bg-primary); }
.toast.error { background: var(--danger); color: var(--bg-primary); }
.toast-enter-active, .toast-leave-active { transition: opacity 0.3s, transform 0.3s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateX(-50%) translateY(-12px); }

/* Profile panel (left) */
.profile-panel {
  width: 280px;
  min-width: 280px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.profile-panel-header {
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.profile-panel-header h3 {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 600;
}

.header-actions { display: flex; gap: 6px; }

.btn-sm { padding: 4px 10px; font-size: 12px; }

.profile-list-loading { padding: 16px; color: var(--text-muted); font-size: 13px; }

.profile-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
}

.profile-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  cursor: pointer;
  border-bottom: 1px solid var(--border);
  transition: background 0.15s;
  gap: 8px;
}

.profile-item:hover { background: var(--bg-hover); }

.profile-item.active {
  background: var(--bg-surface);
  border-left: 3px solid var(--accent);
  padding-left: 13px;
}

.profile-item-main {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.profile-icon {
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.profile-icon.on { color: var(--success); }

.profile-info { flex: 1; min-width: 0; }

.profile-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.profile-desc {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Toggle switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
  cursor: pointer;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--bg-hover);
  border-radius: 20px;
  transition: background 0.2s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  bottom: 2px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: transform 0.2s, background 0.2s;
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--success);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(16px);
  background: #fff;
}

/* Detail area (right) */
.detail-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.detail-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.detail-toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.detail-status {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  background: var(--bg-surface);
  color: var(--text-muted);
  flex-shrink: 0;
}

.detail-status.on {
  background: rgba(166, 227, 161, 0.15);
  color: var(--success);
}

.detail-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.edit-name-input {
  flex: 1;
  max-width: 320px;
  padding: 6px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 500;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
}

.edit-name-input:focus { border-color: var(--accent); }

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

/* Read-only view */
.readonly-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 20px;
  line-height: 1.5;
}

.readonly-entry {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  margin-bottom: 12px;
  overflow: hidden;
}

.readonly-entry-path {
  padding: 8px 14px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.readonly-entry-content {
  padding: 12px 14px;
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 300px;
  overflow-y: auto;
}

.readonly-env-list {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 0;
}

.readonly-env-row {
  padding: 4px 14px;
  font-size: 13px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  display: flex;
  gap: 4px;
}

.env-name { color: var(--warning); }
.env-eq { color: var(--text-muted); }
.env-val { color: var(--success); }

.empty-hint {
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
  padding: 40px 0;
}

/* Form group */
.form-group { margin-bottom: 16px; }

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: var(--text-secondary);
}

.form-group input[type="text"],
.form-group textarea {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 14px;
  font-family: inherit;
  box-sizing: border-box;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input[type="text"]:focus,
.form-group textarea:focus { border-color: var(--accent); }
.form-group textarea { resize: vertical; }

.checkbox-group { display: flex; align-items: center; }

.checkbox-label {
  display: inline-flex !important;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-primary);
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}

/* Sections */
.section { margin-bottom: 28px; }

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-header h4 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

/* Entry card */
.entry-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.entry-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.entry-select {
  flex: 1;
  padding: 6px 10px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  cursor: pointer;
}

.entry-select:focus { border-color: var(--accent); }
.entry-select option { background: var(--bg-secondary); color: var(--text-primary); }

.entry-content {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.6;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

.entry-content:focus { border-color: var(--accent); }

/* Env table */
.env-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 8px;
}

.env-table th {
  text-align: left;
  padding: 8px 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.env-table td {
  padding: 6px 10px;
  border-bottom: 1px solid var(--border);
}

.env-input {
  width: 100%;
  padding: 6px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  outline: none;
  box-sizing: border-box;
}

.env-input:focus { border-color: var(--accent); }
.env-action-cell { width: 40px; text-align: center; }

/* Empty state */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state p { color: var(--text-muted); font-size: 15px; }

/* Modal */
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

.modal-card h3 {
  margin: 0 0 16px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-card p {
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 16px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.import-textarea {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.5;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

.import-textarea:focus { border-color: var(--accent); }
</style>
