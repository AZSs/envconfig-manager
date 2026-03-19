<template>
  <div class="settings-page">
    <h2 class="page-title">设置</h2>

    <!-- 主题 -->
    <section class="settings-section">
      <h3 class="section-title">外观</h3>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">主题模式</span>
          <span class="setting-desc">切换浅色或深色主题</span>
        </div>
        <div class="theme-switcher">
          <button
            class="theme-option"
            :class="{ active: themeStore.mode === 'dark' }"
            @click="themeStore.setMode('dark')"
          >
            <span class="theme-icon">&#127769;</span>
            深色
          </button>
          <button
            class="theme-option"
            :class="{ active: themeStore.mode === 'light' }"
            @click="themeStore.setMode('light')"
          >
            <span class="theme-icon">&#9728;</span>
            浅色
          </button>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">编辑器字体大小</span>
          <span class="setting-desc">调整代码编辑器的字体大小 (10–24px)</span>
        </div>
        <div class="font-size-control">
          <button class="btn btn-ghost size-btn" @click="themeStore.setFontSize(themeStore.editorFontSize - 1)">−</button>
          <span class="font-size-value">{{ themeStore.editorFontSize }}px</span>
          <button class="btn btn-ghost size-btn" @click="themeStore.setFontSize(themeStore.editorFontSize + 1)">+</button>
        </div>
      </div>
    </section>

    <!-- 软件更新 -->
    <section class="settings-section">
      <h3 class="section-title">软件更新</h3>

      <div class="setting-row update-row">
        <div class="setting-info">
          <span class="setting-label">当前版本</span>
          <span class="setting-desc">v{{ appVersion }}</span>
        </div>
        <div class="update-actions">
          <template v-if="updateStatus === 'checking'">
            <span class="update-hint">检查中...</span>
          </template>
          <template v-else-if="updateStatus === 'available'">
            <span class="update-hint update-available">发现新版本 v{{ newVersion }}</span>
            <button class="btn btn-primary" @click="installUpdate">下载更新</button>
          </template>
          <template v-else-if="updateStatus === 'downloading'">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
            </div>
            <span class="update-hint">下载中 {{ downloadProgress }}%</span>
          </template>
          <template v-else-if="updateStatus === 'ready'">
            <span class="update-hint update-available">下载完成，重启生效</span>
            <button class="btn btn-success" @click="relaunch">立即重启</button>
          </template>
          <template v-else-if="updateStatus === 'latest'">
            <span class="update-hint update-latest">已是最新版本</span>
            <button class="btn btn-ghost" @click="checkForUpdate">重新检查</button>
          </template>
          <template v-else-if="updateStatus === 'error'">
            <span class="update-hint update-error">{{ updateError }}</span>
            <button class="btn btn-ghost" @click="checkForUpdate">重试</button>
          </template>
          <template v-else>
            <button class="btn btn-primary" @click="checkForUpdate">检查更新</button>
          </template>
        </div>
      </div>
    </section>

    <!-- 预览 -->
    <section class="settings-section">
      <h3 class="section-title">预览</h3>
      <div class="preview-card">
        <div class="preview-code" :style="{ fontSize: themeStore.editorFontSize + 'px' }">
          <div class="preview-line"><span class="preview-comment"># 示例配置预览</span></div>
          <div class="preview-line"><span class="preview-keyword">export</span> <span class="preview-var">PATH</span>=<span class="preview-string">"/usr/local/bin:$PATH"</span></div>
          <div class="preview-line"><span class="preview-keyword">export</span> <span class="preview-var">EDITOR</span>=<span class="preview-string">"vim"</span></div>
          <div class="preview-line"><span class="preview-keyword">alias</span> <span class="preview-var">ll</span>=<span class="preview-string">"ls -la"</span></div>
        </div>
      </div>
    </section>

    <!-- 关于 -->
    <section class="settings-section">
      <h3 class="section-title">关于</h3>
      <div class="about-info">
        <div class="about-row">
          <span class="about-label">应用名称</span>
          <span class="about-value">EnvConfig Manager</span>
        </div>
        <div class="about-row">
          <span class="about-label">版本</span>
          <span class="about-value">v{{ appVersion }}</span>
        </div>
        <div class="about-row">
          <span class="about-label">技术栈</span>
          <span class="about-value">Tauri 2 + Vue 3 + TypeScript</span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useThemeStore } from '../stores/theme'
import { getVersion } from '@tauri-apps/api/app'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch as appRelaunch } from '@tauri-apps/plugin-process'

const themeStore = useThemeStore()

const appVersion = ref('0.0.0')
const updateStatus = ref<'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'latest' | 'error'>('idle')
const newVersion = ref('')
const downloadProgress = ref(0)
const updateError = ref('')
let pendingUpdate: Update | null = null

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
})

async function checkForUpdate() {
  updateStatus.value = 'checking'
  updateError.value = ''
  try {
    const update = await check()
    if (update) {
      pendingUpdate = update
      newVersion.value = update.version
      updateStatus.value = 'available'
    } else {
      updateStatus.value = 'latest'
    }
  } catch (e) {
    updateError.value = '检查更新失败，请检查网络'
    updateStatus.value = 'error'
    console.error('Update check failed:', e)
  }
}

async function installUpdate() {
  if (!pendingUpdate) return
  updateStatus.value = 'downloading'
  downloadProgress.value = 0
  try {
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started' && event.data.contentLength) {
        downloadProgress.value = 0
      } else if (event.event === 'Progress') {
        downloadProgress.value = Math.min(99, downloadProgress.value + Math.round((event.data.chunkLength / (pendingUpdate as any).contentLength || 1) * 100))
      } else if (event.event === 'Finished') {
        downloadProgress.value = 100
      }
    })
    updateStatus.value = 'ready'
  } catch (e) {
    updateError.value = '下载更新失败'
    updateStatus.value = 'error'
    console.error('Update download failed:', e)
  }
}

async function relaunch() {
  await appRelaunch()
}
</script>

<style scoped>
.settings-page {
  max-width: 640px;
  margin: 0 auto;
}

.page-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 24px;
}

.settings-section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  margin-bottom: 8px;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-muted);
}

/* Theme switcher */
.theme-switcher {
  display: flex;
  gap: 4px;
  background: var(--bg-surface);
  border-radius: 8px;
  padding: 3px;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  background: transparent;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.theme-option.active {
  background: var(--accent);
  color: var(--bg-primary);
}

.theme-icon {
  font-size: 14px;
}

/* Font size control */
.font-size-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.size-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
}

.font-size-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  min-width: 44px;
  text-align: center;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

/* Update section */
.update-row {
  flex-wrap: wrap;
  gap: 12px;
}

.update-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.update-hint {
  font-size: 13px;
  color: var(--text-muted);
}

.update-available {
  color: var(--accent);
  font-weight: 500;
}

.update-latest {
  color: var(--success);
}

.update-error {
  color: var(--danger);
}

.progress-bar {
  width: 120px;
  height: 6px;
  background: var(--bg-surface);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.3s;
}

/* Preview */
.preview-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
}

.preview-code {
  padding: 16px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.7;
}

.preview-line {
  white-space: pre;
}

.preview-comment { color: var(--text-muted); }
.preview-keyword { color: var(--accent); }
.preview-var { color: var(--warning); }
.preview-string { color: var(--success); }

/* About */
.about-info {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 4px 0;
}

.about-row {
  display: flex;
  justify-content: space-between;
  padding: 10px 16px;
}

.about-row + .about-row {
  border-top: 1px solid var(--border);
}

.about-label {
  font-size: 13px;
  color: var(--text-muted);
}

.about-value {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}
</style>
