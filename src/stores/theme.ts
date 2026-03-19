import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { ThemeMode } from '../types'

const STORAGE_KEY = 'envconfig-theme'
const FONT_SIZE_KEY = 'envconfig-editor-font-size'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>(loadMode())
  const editorFontSize = ref(loadFontSize())

  function loadMode(): ThemeMode {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'light' || saved === 'dark') return saved
    return 'dark'
  }

  function loadFontSize(): number {
    const saved = localStorage.getItem(FONT_SIZE_KEY)
    if (saved) {
      const n = parseInt(saved, 10)
      if (n >= 10 && n <= 24) return n
    }
    return 13
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-theme', mode.value)
    localStorage.setItem(STORAGE_KEY, mode.value)
  }

  function applyFontSize() {
    document.documentElement.style.setProperty('--editor-font-size', `${editorFontSize.value}px`)
    localStorage.setItem(FONT_SIZE_KEY, String(editorFontSize.value))
  }

  function toggleMode() {
    mode.value = mode.value === 'dark' ? 'light' : 'dark'
  }

  function setMode(m: ThemeMode) {
    mode.value = m
  }

  function setFontSize(size: number) {
    editorFontSize.value = Math.min(24, Math.max(10, size))
  }

  /** Call once on app startup */
  function init() {
    applyTheme()
    applyFontSize()
  }

  watch(mode, () => applyTheme())
  watch(editorFontSize, () => applyFontSize())

  return { mode, editorFontSize, toggleMode, setMode, setFontSize, init }
})
