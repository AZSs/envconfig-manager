<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1 class="logo">EnvConfig Manager</h1>
        <span class="version">v0.1.0</span>
      </div>
      <nav class="sidebar-nav">
        <router-link to="/config" class="nav-item" active-class="active">
          <span class="nav-icon">&#9881;</span>
          <span>配置文件管理</span>
        </router-link>
        <router-link to="/env" class="nav-item" active-class="active">
          <span class="nav-icon">&#36;</span>
          <span>环境变量管理</span>
        </router-link>
        <router-link to="/backup" class="nav-item" active-class="active">
          <span class="nav-icon">&#128190;</span>
          <span>备份管理</span>
        </router-link>
        <router-link to="/profiles" class="nav-item" active-class="active">
          <span class="nav-icon">&#128260;</span>
          <span>配置集管理</span>
        </router-link>
        <router-link to="/settings" class="nav-item" active-class="active">
          <span class="nav-icon">&#9879;</span>
          <span>设置</span>
        </router-link>
      </nav>
      <div class="sidebar-footer">
        <span class="platform-info">{{ platform }}</span>
      </div>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useThemeStore } from './stores/theme'

const themeStore = useThemeStore()

onMounted(() => {
  themeStore.init()
})

const platform = computed(() => {
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('mac')) return 'macOS'
  if (ua.includes('win')) return 'Windows'
  if (ua.includes('linux')) return 'Linux'
  return 'Unknown'
})
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  user-select: none;
}

.sidebar-header {
  padding: 20px 16px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.logo {
  font-size: 18px;
  font-weight: 700;
  color: var(--accent);
}

.version {
  font-size: 11px;
  color: var(--text-muted);
}

.sidebar-nav {
  flex: 1;
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 13px;
  transition: all 0.15s;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: rgba(137, 180, 250, 0.12);
  color: var(--accent);
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border);
}

.platform-info {
  font-size: 11px;
  color: var(--text-muted);
}

.main-content {
  flex: 1;
  overflow: auto;
  padding: 24px;
}
</style>
