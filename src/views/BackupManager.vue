<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useBackupStore } from '../stores/backup'
import type { BackupSnapshot } from '../types'

const backupStore = useBackupStore()

// Modals
const showCreateModal = ref(false)
const showRestoreModal = ref(false)
const showDeleteModal = ref(false)

// Create backup form
const createForm = ref({ sourcePath: '', remark: '' })

// Restore backup form
const restoreTarget = ref('')
const restoreSnapshot = ref<BackupSnapshot | null>(null)

// Delete target
const deleteSnapshot = ref<BackupSnapshot | null>(null)

// Sorted snapshots (newest first)
const sortedSnapshots = computed(() => {
  return [...backupStore.snapshots].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  )
})

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function formatTime(timestamp: string): string {
  const d = new Date(timestamp)
  return d.toLocaleString()
}

// Create backup
async function handleCreate() {
  if (!createForm.value.sourcePath) return
  try {
    await backupStore.createBackup(createForm.value.sourcePath, createForm.value.remark)
    showCreateModal.value = false
    createForm.value = { sourcePath: '', remark: '' }
  } catch (e) {
    alert('创建备份失败: ' + e)
  }
}

// Restore backup
function openRestore(snapshot: BackupSnapshot) {
  restoreSnapshot.value = snapshot
  restoreTarget.value = ''
  showRestoreModal.value = true
}

async function handleRestore() {
  if (!restoreSnapshot.value || !restoreTarget.value) return
  try {
    await backupStore.restoreBackup(restoreSnapshot.value.id, restoreTarget.value)
    showRestoreModal.value = false
  } catch (e) {
    alert('回滚失败: ' + e)
  }
}

// Delete backup
function openDelete(snapshot: BackupSnapshot) {
  deleteSnapshot.value = snapshot
  showDeleteModal.value = true
}

async function handleDelete() {
  if (!deleteSnapshot.value) return
  try {
    await backupStore.deleteBackup(deleteSnapshot.value.id)
    showDeleteModal.value = false
  } catch (e) {
    alert('删除失败: ' + e)
  }
}

onMounted(() => {
  backupStore.loadConfig()
  backupStore.loadBackups()
})
</script>

<template>
  <div class="backup-manager">
    <!-- Top info bar -->
    <div class="info-bar">
      <div class="info-items">
        <div class="info-item">
          <span class="info-label">备份目录</span>
          <span class="info-value">{{ backupStore.config?.backupDir || '--' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">快照数量</span>
          <span class="info-value">{{ backupStore.snapshots.length }} / {{ backupStore.config?.maxSnapshots ?? '--' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">容量上限</span>
          <span class="info-value">{{ backupStore.config?.maxSizeMB ?? '--' }} MB</span>
        </div>
      </div>
      <button class="btn btn-primary" @click="showCreateModal = true">手动备份</button>
    </div>

    <!-- Loading -->
    <div v-if="backupStore.loading" class="loading">加载中...</div>

    <!-- Empty state -->
    <div v-else-if="sortedSnapshots.length === 0" class="empty-state">
      <div class="empty-icon">📁</div>
      <p>暂无备份快照</p>
      <p class="text-muted">点击「手动备份」创建第一个快照</p>
    </div>

    <!-- Snapshot list -->
    <div v-else class="snapshot-list">
      <div
        v-for="snapshot in sortedSnapshots"
        :key="snapshot.id"
        class="snapshot-card"
      >
        <div class="snapshot-info">
          <div class="snapshot-header">
            <span class="snapshot-name">{{ snapshot.fileName }}</span>
            <span class="snapshot-size">{{ formatFileSize(snapshot.fileSize) }}</span>
          </div>
          <div class="snapshot-meta">
            <span class="snapshot-time">{{ formatTime(snapshot.timestamp) }}</span>
            <span v-if="snapshot.remark" class="snapshot-remark">{{ snapshot.remark }}</span>
          </div>
        </div>
        <div class="snapshot-actions">
          <button class="btn btn-success" @click="openRestore(snapshot)">回滚</button>
          <button class="btn btn-danger" @click="openDelete(snapshot)">删除</button>
        </div>
      </div>
    </div>

    <!-- Create backup modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal-card">
        <h3>手动备份</h3>
        <div class="form-group">
          <label>配置文件路径</label>
          <input
            v-model="createForm.sourcePath"
            type="text"
            placeholder="请输入要备份的文件路径"
          />
        </div>
        <div class="form-group">
          <label>备注</label>
          <textarea
            v-model="createForm.remark"
            placeholder="可选：填写备注信息"
            rows="3"
          />
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showCreateModal = false">取消</button>
          <button class="btn btn-primary" @click="handleCreate">确认备份</button>
        </div>
      </div>
    </div>

    <!-- Restore modal -->
    <div v-if="showRestoreModal" class="modal-overlay" @click.self="showRestoreModal = false">
      <div class="modal-card">
        <h3>回滚确认</h3>
        <div class="warning-box">
          回滚将覆盖当前配置文件，系统会自动创建当前配置的备份
        </div>
        <div class="restore-info">
          <span class="info-label">快照文件</span>
          <span class="info-value">{{ restoreSnapshot?.fileName }}</span>
        </div>
        <div class="form-group">
          <label>目标文件路径</label>
          <input
            v-model="restoreTarget"
            type="text"
            placeholder="请输入回滚的目标文件路径"
          />
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showRestoreModal = false">取消</button>
          <button class="btn btn-success" @click="handleRestore">确认回滚</button>
        </div>
      </div>
    </div>

    <!-- Delete modal -->
    <div v-if="showDeleteModal" class="modal-overlay" @click.self="showDeleteModal = false">
      <div class="modal-card">
        <h3>删除确认</h3>
        <p>确定要删除快照「{{ deleteSnapshot?.fileName }}」吗？此操作不可撤销。</p>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="showDeleteModal = false">取消</button>
          <button class="btn btn-danger" @click="handleDelete">确认删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-manager {
  padding: 24px;
  color: var(--text-primary, #cdd6f4);
  min-height: 100%;
}

/* Info bar */
.info-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-surface, #313244);
  border-radius: 8px;
  padding: 16px 20px;
  margin-bottom: 24px;
}

.info-items {
  display: flex;
  gap: 32px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 12px;
  color: var(--text-muted, #6c7086);
}

.info-value {
  font-size: 14px;
  color: var(--text-primary, #cdd6f4);
}

/* Loading */
.loading {
  text-align: center;
  padding: 48px 0;
  color: var(--text-secondary, #a6adc8);
}

/* Empty state */
.empty-state {
  text-align: center;
  padding: 64px 0;
  color: var(--text-secondary, #a6adc8);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state .text-muted {
  color: var(--text-muted, #6c7086);
  font-size: 13px;
}

/* Snapshot list */
.snapshot-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.snapshot-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-surface, #313244);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  padding: 16px;
  transition: border-color 0.2s;
}

.snapshot-card:hover {
  border-color: var(--accent, #89b4fa);
}

.snapshot-info {
  flex: 1;
  min-width: 0;
}

.snapshot-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 6px;
}

.snapshot-name {
  font-weight: 500;
  color: var(--text-primary, #cdd6f4);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.snapshot-size {
  font-size: 12px;
  color: var(--text-muted, #6c7086);
  white-space: nowrap;
}

.snapshot-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 13px;
  color: var(--text-secondary, #a6adc8);
}

.snapshot-time {
  color: var(--text-muted, #6c7086);
}

.snapshot-remark {
  color: var(--text-secondary, #a6adc8);
}

.snapshot-actions {
  display: flex;
  gap: 8px;
  margin-left: 16px;
  flex-shrink: 0;
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card {
  background: var(--bg-surface, #313244);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  padding: 24px;
  width: 480px;
  max-width: 90vw;
}

.modal-card h3 {
  margin: 0 0 16px;
  font-size: 18px;
  color: var(--text-primary, #cdd6f4);
}

.modal-card p {
  color: var(--text-secondary, #a6adc8);
  line-height: 1.6;
}

.warning-box {
  background: rgba(249, 226, 175, 0.1);
  border: 1px solid var(--warning, #f9e2af);
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 16px;
  color: var(--warning, #f9e2af);
  font-size: 13px;
  line-height: 1.5;
}

.restore-info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  font-size: 14px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: var(--text-secondary, #a6adc8);
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-secondary, #181825);
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  color: var(--text-primary, #cdd6f4);
  font-size: 14px;
  font-family: inherit;
  box-sizing: border-box;
  outline: none;
  transition: border-color 0.2s;
}

.form-group input:focus,
.form-group textarea:focus {
  border-color: var(--accent, #89b4fa);
}

.form-group textarea {
  resize: vertical;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
