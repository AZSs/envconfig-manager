import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { EnvVariable } from '../types'

export const useEnvVarStore = defineStore('envVar', () => {
  const variables = ref<EnvVariable[]>([])
  const loading = ref(false)
  const keyword = ref('')

  async function loadVariables() {
    loading.value = true
    try {
      variables.value = await invoke<EnvVariable[]>('get_env_variables')
    } catch (e) {
      console.error('获取环境变量失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function searchVariables(kw: string) {
    keyword.value = kw
    if (!kw.trim()) {
      return loadVariables()
    }
    loading.value = true
    try {
      variables.value = await invoke<EnvVariable[]>('search_env_variables', { keyword: kw })
    } catch (e) {
      console.error('搜索失败:', e)
    } finally {
      loading.value = false
    }
  }

  async function setVariable(name: string, value: string, scope: string) {
    try {
      await invoke('set_env_variable', { name, value, scope })
      await loadVariables()
    } catch (e) {
      console.error('设置环境变量失败:', e)
      throw e
    }
  }

  async function deleteVariable(name: string, scope: string) {
    try {
      await invoke('delete_env_variable', { name, scope })
      await loadVariables()
    } catch (e) {
      console.error('删除环境变量失败:', e)
      throw e
    }
  }

  return { variables, loading, keyword, loadVariables, searchVariables, setVariable, deleteVariable }
})
