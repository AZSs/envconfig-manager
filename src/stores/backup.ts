import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { BackupSnapshot, BackupConfig } from '../types'

export const useBackupStore = defineStore('backup', () => {
  const snapshots = ref<BackupSnapshot[]>([])
  const config = ref<BackupConfig | null>(null)
  const loading = ref(false)

  async function loadBackups() {
    loading.value = true
    try {
      snapshots.value = await invoke<BackupSnapshot[]>('list_backups')
    } catch (e) {
      console.error('加载备份列表失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function createBackup(sourcePath: string, remark: string) {
    try {
      const snapshot = await invoke<BackupSnapshot>('create_backup', { sourcePath, remark })
      snapshots.value.push(snapshot)
      return snapshot
    } catch (e) {
      console.error('创建备份失败:', e)
      throw e
    }
  }

  async function restoreBackup(backupId: string, targetPath: string) {
    try {
      await invoke('restore_backup', { backupId, targetPath })
      await loadBackups()
    } catch (e) {
      console.error('回滚失败:', e)
      throw e
    }
  }

  async function deleteBackup(backupId: string) {
    try {
      await invoke('delete_backup', { backupId })
      snapshots.value = snapshots.value.filter(s => s.id !== backupId)
    } catch (e) {
      console.error('删除备份失败:', e)
      throw e
    }
  }

  async function loadConfig() {
    try {
      config.value = await invoke<BackupConfig>('get_backup_config')
    } catch (e) {
      console.error('加载备份配置失败:', e)
    }
  }

  return { snapshots, config, loading, loadBackups, createBackup, restoreBackup, deleteBackup, loadConfig }
})
