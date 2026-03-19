import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ValidationResult } from '../types'

export const useValidationStore = defineStore('validation', () => {
  const lastResult = ref<ValidationResult | null>(null)
  const loading = ref(false)

  let syntaxTimer: ReturnType<typeof setTimeout> | null = null

  async function validateFile(path: string, content: string) {
    loading.value = true
    try {
      const result = await invoke<ValidationResult>('validate_config_file', { path, content })
      lastResult.value = result
      return result
    } catch (e) {
      console.error('校验配置文件失败:', e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function checkSyntaxLive(path: string, content: string) {
    try {
      const result = await invoke<ValidationResult>('check_syntax', { path, content })
      lastResult.value = result
    } catch (e) {
      console.error('语法检查失败:', e)
    }
  }

  function debouncedSyntaxCheck(path: string, content: string) {
    if (syntaxTimer) {
      clearTimeout(syntaxTimer)
    }
    syntaxTimer = setTimeout(() => {
      checkSyntaxLive(path, content)
    }, 500)
  }

  function clearValidation() {
    lastResult.value = null
  }

  return { lastResult, loading, validateFile, checkSyntaxLive, debouncedSyntaxCheck, clearValidation }
})
