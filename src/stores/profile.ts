import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConfigProfile, ConfigProfileEntry, EnvVarEntry, ProfileDiffResult } from '../types'

export const useProfileStore = defineStore('profile', () => {
  const profiles = ref<ConfigProfile[]>([])
  const loading = ref(false)

  async function loadProfiles() {
    loading.value = true
    try {
      profiles.value = await invoke<ConfigProfile[]>('list_profiles')
    } catch (e) {
      console.error('加载配置集失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function createProfile(name: string, description: string, entries: ConfigProfileEntry[], envVars: EnvVarEntry[]) {
    try {
      const profile = await invoke<ConfigProfile>('create_profile', { name, description, entries, envVars })
      await loadProfiles()
      return profile
    } catch (e) {
      console.error('创建配置集失败:', e)
      throw e
    }
  }

  async function updateProfile(id: string, name: string, description: string, entries: ConfigProfileEntry[], envVars: EnvVarEntry[]) {
    try {
      const profile = await invoke<ConfigProfile>('update_profile', { id, name, description, entries, envVars })
      await loadProfiles()
      return profile
    } catch (e) {
      console.error('更新配置集失败:', e)
      throw e
    }
  }

  async function deleteProfile(id: string) {
    try {
      await invoke('delete_profile', { id })
      await loadProfiles()
    } catch (e) {
      console.error('删除配置集失败:', e)
      throw e
    }
  }

  async function applyProfile(id: string) {
    try {
      const result = await invoke<string[]>('apply_profile', { id })
      return result
    } catch (e) {
      console.error('应用配置集失败:', e)
      throw e
    }
  }

  async function getDiff(id: string) {
    try {
      const result = await invoke<ProfileDiffResult>('diff_profile', { id })
      return result
    } catch (e) {
      console.error('获取配置集差异失败:', e)
      throw e
    }
  }

  async function exportProfile(id: string) {
    try {
      const result = await invoke<string>('export_profile', { id })
      return result
    } catch (e) {
      console.error('导出配置集失败:', e)
      throw e
    }
  }

  async function importProfile(jsonData: string) {
    try {
      const profile = await invoke<ConfigProfile>('import_profile', { jsonData })
      await loadProfiles()
      return profile
    } catch (e) {
      console.error('导入配置集失败:', e)
      throw e
    }
  }

  async function toggleProfile(id: string, active: boolean) {
    try {
      const result = await invoke<string[]>('toggle_profile', { id, active })
      await loadProfiles()
      return result
    } catch (e) {
      console.error('切换配置集失败:', e)
      throw e
    }
  }

  return { profiles, loading, loadProfiles, createProfile, updateProfile, deleteProfile, applyProfile, getDiff, exportProfile, importProfile, toggleProfile }
})
