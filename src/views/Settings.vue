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
  </div>
</template>

<script setup lang="ts">
import { useThemeStore } from '../stores/theme'

const themeStore = useThemeStore()
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

.preview-comment {
  color: var(--text-muted);
}

.preview-keyword {
  color: var(--accent);
}

.preview-var {
  color: var(--warning);
}

.preview-string {
  color: var(--success);
}
</style>
