<template>
  <div v-if="visible" class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal-card">
      <h3 class="modal-title">配置差异对比</h3>

      <!-- Tab bar -->
      <div v-if="diffs.length > 1" class="tab-bar">
        <button
          v-for="(diff, idx) in diffs"
          :key="diff.filePath"
          class="tab-item"
          :class="{ active: activeTab === idx }"
          @click="activeTab = idx"
        >
          {{ fileName(diff.filePath) }}
        </button>
      </div>

      <!-- Diff content -->
      <div class="diff-content">
        <template v-if="diffResult.length > 0 && hasDiff">
          <div
            v-for="(part, idx) in diffResult"
            :key="idx"
            class="diff-line"
            :class="{
              'diff-added': part.added,
              'diff-removed': part.removed,
            }"
          >
            <span class="line-num">{{ part.leftNum ?? '' }}</span>
            <span class="line-num">{{ part.rightNum ?? '' }}</span>
            <span class="diff-prefix">{{ part.added ? '+' : part.removed ? '-' : ' ' }}</span>
            <span class="diff-text">{{ part.text }}</span>
          </div>
        </template>
        <template v-else-if="diffResult.length > 0">
          <div
            v-for="(part, idx) in diffResult"
            :key="idx"
            class="diff-line"
          >
            <span class="line-num">{{ part.leftNum }}</span>
            <span class="diff-text">{{ part.text }}</span>
          </div>
          <div class="no-diff-hint">内容一致，无差异</div>
        </template>
        <div v-else class="no-diff">无差异</div>
      </div>

      <!-- Footer -->
      <div class="modal-actions">
        <button class="btn btn-ghost" @click="$emit('cancel')">取消</button>
        <button class="btn btn-primary" @click="$emit('confirm')">确认应用</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { diffLines } from 'diff'
import type { ProfileDiffItem } from '../types'

interface DiffLineEntry {
  text: string
  added: boolean
  removed: boolean
  leftNum: number | null
  rightNum: number | null
}

const props = defineProps<{
  visible: boolean
  diffs: ProfileDiffItem[]
}>()

defineEmits<{
  confirm: []
  cancel: []
}>()

const activeTab = ref(0)

// Reset tab when modal opens or diffs change
watch(
  () => props.visible,
  (val) => {
    if (val) activeTab.value = 0
  }
)

const currentDiff = computed(() => {
  if (props.diffs.length === 0) return null
  return props.diffs[activeTab.value] ?? props.diffs[0]
})

const diffResult = computed<DiffLineEntry[]>(() => {
  if (!currentDiff.value) return []

  const changes = diffLines(
    currentDiff.value.currentContent,
    currentDiff.value.profileContent
  )

  const entries: DiffLineEntry[] = []
  let leftLine = 1
  let rightLine = 1

  for (const change of changes) {
    const lines = change.value.replace(/\n$/, '').split('\n')
    for (const line of lines) {
      if (change.added) {
        entries.push({
          text: line,
          added: true,
          removed: false,
          leftNum: null,
          rightNum: rightLine++,
        })
      } else if (change.removed) {
        entries.push({
          text: line,
          added: false,
          removed: true,
          leftNum: leftLine++,
          rightNum: null,
        })
      } else {
        entries.push({
          text: line,
          added: false,
          removed: false,
          leftNum: leftLine++,
          rightNum: rightLine++,
        })
      }
    }
  }

  return entries
})

const hasDiff = computed(() => {
  return diffResult.value.some(entry => entry.added || entry.removed)
})

function fileName(filePath: string): string {
  return filePath.split('/').pop() ?? filePath
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.modal-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
  width: 720px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: var(--text-primary);
}

/* Tab bar */
.tab-bar {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
  border-bottom: 1px solid var(--border);
  padding-bottom: 0;
}

.tab-item {
  padding: 6px 14px;
  font-size: 13px;
  color: var(--text-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
  margin-bottom: -1px;
}

.tab-item:hover {
  color: var(--text-primary);
}

.tab-item.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* Diff content */
.diff-content {
  flex: 1;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-primary);
  margin-bottom: 16px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.diff-line {
  display: flex;
  align-items: baseline;
  padding: 0 8px;
  min-height: 22px;
}

.diff-added {
  background: rgba(166, 227, 161, 0.15);
}

.diff-removed {
  background: rgba(243, 139, 168, 0.15);
}

.line-num {
  display: inline-block;
  width: 40px;
  flex-shrink: 0;
  text-align: right;
  padding-right: 8px;
  color: var(--text-muted);
  font-size: 12px;
  user-select: none;
}

.diff-prefix {
  flex-shrink: 0;
  width: 16px;
  color: var(--text-muted);
  user-select: none;
}

.diff-text {
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
}

.no-diff {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 14px;
}

.no-diff-hint {
  padding: 8px 16px;
  text-align: center;
  color: var(--success);
  font-size: 12px;
  background: rgba(166, 227, 161, 0.08);
}

/* Footer actions */
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
