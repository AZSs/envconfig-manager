import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConfigFile } from '../types'

export const useConfigStore = defineStore('config', () => {
  const files = ref<ConfigFile[]>([])
  const currentFile = ref<ConfigFile | null>(null)
  const loading = ref(false)
  const originalContent = ref('')
  const dirty = ref(false)

  async function scanFiles() {
    loading.value = true
    try {
      files.value = await invoke<ConfigFile[]>('scan_config_files')
    } catch (e) {
      console.error('扫描配置文件失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function readFile(path: string) {
    try {
      const content = await invoke<string>('read_config_file', { path })
      const file = files.value.find(f => f.path === path)
      if (file) {
        currentFile.value = { ...file, content }
        originalContent.value = content
      }
    } catch (e) {
      console.error('读取文件失败:', e)
    }
  }

  async function saveFile(path: string, content: string) {
    try {
      await invoke('write_config_file', { path, content })
      if (currentFile.value) {
        currentFile.value.content = content
      }
    } catch (e) {
      console.error('保存文件失败:', e)
      throw e
    }
  }

  async function applyFile(path: string) {
    try {
      const result = await invoke<string>('apply_config_file', { path })
      return result
    } catch (e) {
      console.error('生效失败:', e)
      throw e
    }
  }

  function setDirty(isDirty: boolean) {
    dirty.value = isDirty
  }

  function getOriginalContent() {
    return originalContent.value
  }

  return { files, currentFile, loading, originalContent, dirty, scanFiles, readFile, saveFile, applyFile, setDirty, getOriginalContent }
})
