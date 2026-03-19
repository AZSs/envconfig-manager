<template>
  <div v-if="result && result.issues.length > 0" class="validation-panel">
    <div class="summary-bar">
      <span class="summary-errors">{{ errorCount }} 个错误</span>,
      <span class="summary-warnings">{{ warningCount }} 个警告</span>
    </div>
    <ul class="issue-list">
      <li v-for="(issue, idx) in result.issues" :key="idx" class="issue-item">
        <span v-if="issue.severity === 'error'" class="severity-icon severity-error">●</span>
        <span v-else class="severity-icon severity-warning">▲</span>
        <span
          v-if="issue.line !== null"
          class="line-number"
          @click="$emit('gotoLine', issue.line!)"
        >第 {{ issue.line }} 行</span>
        <span class="category-badge">{{ categoryLabel(issue.category) }}</span>
        <span class="issue-message">{{ issue.message }}</span>
        <span v-if="issue.suggestion" class="issue-suggestion">{{ issue.suggestion }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ValidationResult } from '../types'

const props = defineProps<{
  result: ValidationResult | null
}>()

defineEmits<{
  gotoLine: [line: number]
}>()

const errorCount = computed(() =>
  props.result?.issues.filter(i => i.severity === 'error').length ?? 0
)

const warningCount = computed(() =>
  props.result?.issues.filter(i => i.severity === 'warning').length ?? 0
)

function categoryLabel(category: 'syntax' | 'path' | 'conflict'): string {
  const map: Record<string, string> = {
    syntax: '语法',
    path: '路径',
    conflict: '冲突',
  }
  return `[${map[category] ?? category}]`
}
</script>

<style scoped>
.validation-panel {
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  max-height: 200px;
  overflow-y: auto;
  color: var(--text-primary);
}

.summary-bar {
  padding: 8px 16px;
  font-size: 13px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface);
}

.summary-errors {
  color: var(--danger);
  font-weight: 600;
}

.summary-warnings {
  color: #fab387;
  font-weight: 600;
}

.issue-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.issue-item {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 6px 16px;
  font-size: 13px;
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}

.severity-icon {
  flex-shrink: 0;
  font-size: 12px;
}

.severity-error {
  color: var(--danger);
}

.severity-warning {
  color: #fab387;
}

.line-number {
  cursor: pointer;
  color: var(--accent);
  white-space: nowrap;
}

.line-number:hover {
  text-decoration: underline;
}

.category-badge {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.issue-message {
  color: var(--text-primary);
}

.issue-suggestion {
  color: var(--text-muted);
  font-size: 12px;
  width: 100%;
  padding-left: 20px;
}
</style>
