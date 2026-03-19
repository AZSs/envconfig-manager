<template>
  <div class="env-page">
    <!-- 顶部操作栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <input
          class="input search-input"
          type="text"
          placeholder="搜索变量名或值..."
          v-model="searchKeyword"
          @input="onSearch"
        />
        <div class="filter-group">
          <button
            class="btn btn-ghost filter-btn"
            :class="{ 'filter-active': scopeFilter === 'all' }"
            @click="scopeFilter = 'all'"
          >全部</button>
          <button
            class="btn btn-ghost filter-btn"
            :class="{ 'filter-active': scopeFilter === 'user' }"
            @click="scopeFilter = 'user'"
          >用户级</button>
          <button
            class="btn btn-ghost filter-btn"
            :class="{ 'filter-active': scopeFilter === 'system' }"
            @click="scopeFilter = 'system'"
          >系统级</button>
        </div>
        <div class="view-group">
          <button
            class="btn btn-ghost filter-btn"
            :class="{ 'filter-active': viewMode === 'flat' }"
            @click="viewMode = 'flat'"
          >列表</button>
          <button
            class="btn btn-ghost filter-btn"
            :class="{ 'filter-active': viewMode === 'grouped' }"
            @click="viewMode = 'grouped'"
          >分组</button>
        </div>
      </div>
      <button class="btn btn-primary" @click="openAddModal">+ 新增变量</button>
    </div>

    <!-- 列表模式 -->
    <div v-if="viewMode === 'flat'" class="table-wrapper">
      <div class="table-header">
        <div class="col col-name sortable" @click="toggleSort">
          变量名
          <span class="sort-icon">{{ sortAsc ? '▲' : '▼' }}</span>
        </div>
        <div class="col col-value">值</div>
        <div class="col col-scope">范围</div>
        <div class="col col-actions">操作</div>
      </div>
      <div v-if="envVarStore.loading" class="table-empty">加载中...</div>
      <div v-else-if="filteredVariables.length === 0" class="table-empty">暂无数据</div>
      <div
        v-for="item in filteredVariables"
        :key="item.name + item.scope"
        class="table-row"
      >
        <div class="col col-name mono">{{ item.name }}</div>
        <div class="col col-value" :title="item.value">
          <span class="value-text mono">{{ item.value }}</span>
        </div>
        <div class="col col-scope">
          <span :class="['badge', item.scope === 'system' ? 'badge-system' : 'badge-user']">
            {{ item.scope === 'system' ? '系统' : '用户' }}
          </span>
        </div>
        <div class="col col-actions">
          <button class="btn btn-ghost btn-sm" @click="openEditModal(item)">编辑</button>
          <button class="btn btn-danger btn-sm" @click="confirmDelete(item)">删除</button>
        </div>
      </div>
    </div>

    <!-- 分组模式 -->
    <div v-else class="groups-wrapper">
      <div v-if="envVarStore.loading" class="table-empty">加载中...</div>
      <div v-else-if="groupedVariables.length === 0" class="table-empty">暂无数据</div>
      <div v-for="group in groupedVariables" :key="group.prefix" class="group-section">
        <div class="group-header" @click="toggleGroup(group.prefix)">
          <span class="group-toggle">{{ expandedGroups.has(group.prefix) ? '▼' : '▶' }}</span>
          <span class="group-name">{{ group.prefix }}</span>
          <span class="group-count">{{ group.items.length }} 个变量</span>
        </div>
        <div v-if="expandedGroups.has(group.prefix)" class="group-body">
          <div
            v-for="item in group.items"
            :key="item.name + item.scope"
            class="table-row"
          >
            <div class="col col-name mono">{{ item.name }}</div>
            <div class="col col-value" :title="item.value">
              <span class="value-text mono">{{ item.value }}</span>
            </div>
            <div class="col col-scope">
              <span :class="['badge', item.scope === 'system' ? 'badge-system' : 'badge-user']">
                {{ item.scope === 'system' ? '系统' : '用户' }}
              </span>
            </div>
            <div class="col col-actions">
              <button class="btn btn-ghost btn-sm" @click="openEditModal(item)">编辑</button>
              <button class="btn btn-danger btn-sm" @click="confirmDelete(item)">删除</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新增/编辑弹窗 -->
    <div v-if="showFormModal" class="modal-overlay" @click.self="closeFormModal">
      <div class="modal-card">
        <h3 class="modal-title">{{ isEditing ? '编辑变量' : '新增变量' }}</h3>
        <div class="form-group">
          <label class="form-label">变量名</label>
          <input
            class="input form-input"
            v-model="formData.name"
            :disabled="isEditing"
            placeholder="例如 MY_VARIABLE"
          />
          <span v-if="formErrors.name" class="form-error">{{ formErrors.name }}</span>
        </div>
        <div class="form-group">
          <label class="form-label">值</label>
          <textarea
            class="input form-input form-textarea"
            v-model="formData.value"
            rows="3"
            placeholder="变量值"
          ></textarea>
        </div>
        <div class="form-group">
          <label class="form-label">生效范围</label>
          <select class="input form-input" v-model="formData.scope">
            <option value="user">用户级</option>
            <option value="system">系统级</option>
          </select>
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="closeFormModal">取消</button>
          <button class="btn btn-primary" @click="submitForm">确认</button>
        </div>
      </div>
    </div>

    <!-- 删除确认弹窗 -->
    <div v-if="showDeleteModal" class="modal-overlay" @click.self="closeDeleteModal">
      <div class="modal-card">
        <h3 class="modal-title">确认删除</h3>
        <p class="modal-text">
          确定要删除变量 <strong class="mono">{{ deleteTarget?.name }}</strong> 吗？
        </p>
        <p v-if="deleteTarget?.scope === 'system'" class="modal-warning">
          警告：这是一个系统级变量，删除后可能影响系统正常运行！
        </p>
        <div class="modal-actions">
          <button class="btn btn-ghost" @click="closeDeleteModal">取消</button>
          <button class="btn btn-danger" @click="doDelete">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from 'vue'
import { useEnvVarStore } from '../stores/envVar'
import type { EnvVariable } from '../types'

const envVarStore = useEnvVarStore()

// 搜索、筛选、视图
const searchKeyword = ref('')
const scopeFilter = ref<'all' | 'user' | 'system'>('all')
const sortAsc = ref(true)
const viewMode = ref<'flat' | 'grouped'>('grouped')
const expandedGroups = reactive(new Set<string>())

function onSearch() {
  envVarStore.searchVariables(searchKeyword.value)
}

const filteredVariables = computed(() => {
  let list = [...envVarStore.variables]
  if (scopeFilter.value !== 'all') {
    list = list.filter(v => v.scope === scopeFilter.value)
  }
  list.sort((a, b) => {
    const cmp = a.name.localeCompare(b.name)
    return sortAsc.value ? cmp : -cmp
  })
  return list
})

/** 提取变量前缀作为分组名 */
function getPrefix(name: string): string {
  // 按 _ 拆分，取第一段作为前缀；单个词的归入 "其他"
  const parts = name.split('_')
  if (parts.length <= 1) return '其他'
  return parts[0].toUpperCase()
}

interface VarGroup {
  prefix: string
  items: EnvVariable[]
}

const groupedVariables = computed<VarGroup[]>(() => {
  const map = new Map<string, EnvVariable[]>()
  for (const v of filteredVariables.value) {
    const prefix = getPrefix(v.name)
    if (!map.has(prefix)) map.set(prefix, [])
    map.get(prefix)!.push(v)
  }
  const groups: VarGroup[] = []
  for (const [prefix, items] of map) {
    groups.push({ prefix, items })
  }
  // 按组内变量数量降序排列，让大分组靠前
  groups.sort((a, b) => b.items.length - a.items.length)
  return groups
})

function toggleGroup(prefix: string) {
  if (expandedGroups.has(prefix)) {
    expandedGroups.delete(prefix)
  } else {
    expandedGroups.add(prefix)
  }
}

function toggleSort() {
  sortAsc.value = !sortAsc.value
}

// 表单弹窗
const showFormModal = ref(false)
const isEditing = ref(false)
const formData = ref({ name: '', value: '', scope: 'user' as string })
const formErrors = ref({ name: '' })

function openAddModal() {
  isEditing.value = false
  formData.value = { name: '', value: '', scope: 'user' }
  formErrors.value = { name: '' }
  showFormModal.value = true
}

function openEditModal(item: EnvVariable) {
  isEditing.value = true
  formData.value = { name: item.name, value: item.value, scope: item.scope }
  formErrors.value = { name: '' }
  showFormModal.value = true
}

function closeFormModal() {
  showFormModal.value = false
}

function validateForm(): boolean {
  formErrors.value.name = ''
  const name = formData.value.name.trim()
  if (!name) {
    formErrors.value.name = '变量名不能为空'
    return false
  }
  if (/[\s=]/.test(name)) {
    formErrors.value.name = '变量名不能包含空格或等号'
    return false
  }
  return true
}

async function submitForm() {
  if (!validateForm()) return
  try {
    await envVarStore.setVariable(formData.value.name.trim(), formData.value.value, formData.value.scope)
    closeFormModal()
  } catch {
    // store 已打印错误
  }
}

// 删除弹窗
const showDeleteModal = ref(false)
const deleteTarget = ref<EnvVariable | null>(null)

function confirmDelete(item: EnvVariable) {
  deleteTarget.value = item
  showDeleteModal.value = true
}

function closeDeleteModal() {
  showDeleteModal.value = false
  deleteTarget.value = null
}

async function doDelete() {
  if (!deleteTarget.value) return
  try {
    await envVarStore.deleteVariable(deleteTarget.value.name, deleteTarget.value.scope)
    closeDeleteModal()
  } catch {
    // store 已打印错误
  }
}

onMounted(() => {
  envVarStore.loadVariables()
})
</script>

<style scoped>
.env-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 20px;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.search-input {
  width: 260px;
}

.filter-group,
.view-group {
  display: flex;
  gap: 4px;
}

.view-group {
  margin-left: 8px;
  padding-left: 12px;
  border-left: 1px solid var(--border);
}

.filter-btn {
  padding: 5px 12px;
  font-size: 12px;
}

.filter-active {
  background: rgba(137, 180, 250, 0.12);
  color: var(--accent);
  border-color: var(--accent);
}

/* 表格（列表模式） */
.table-wrapper {
  flex: 1;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-secondary);
}

.table-header {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  position: sticky;
  top: 0;
  z-index: 1;
}

.table-row {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border);
  transition: background 0.15s;
}

.table-row:last-child {
  border-bottom: none;
}

.table-row:hover {
  background: var(--bg-hover);
}

.col {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.col-name {
  flex: 2;
  min-width: 0;
}

.col-value {
  flex: 3;
  min-width: 0;
}

.col-scope {
  flex: 0 0 80px;
  text-align: center;
}

.col-actions {
  flex: 0 0 140px;
  display: flex;
  gap: 6px;
  justify-content: flex-end;
}

.mono {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
}

.value-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.sortable {
  cursor: pointer;
  user-select: none;
}

.sort-icon {
  font-size: 10px;
  margin-left: 4px;
  color: var(--text-muted);
}

.table-empty {
  padding: 40px;
  text-align: center;
  color: var(--text-muted);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

/* 分组模式 */
.groups-wrapper {
  flex: 1;
  overflow: auto;
}

.group-section {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-secondary);
  margin-bottom: 8px;
  overflow: hidden;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  cursor: pointer;
  user-select: none;
  background: var(--bg-surface);
  transition: background 0.15s;
}

.group-header:hover {
  background: var(--bg-hover);
}

.group-toggle {
  font-size: 10px;
  color: var(--text-muted);
  width: 14px;
}

.group-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
}

.group-count {
  font-size: 11px;
  color: var(--text-muted);
  margin-left: auto;
}

.group-body {
  border-top: 1px solid var(--border);
}

/* 弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
  width: 440px;
  max-width: 90vw;
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 20px;
  color: var(--text-primary);
}

.modal-text {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin-bottom: 12px;
}

.modal-warning {
  font-size: 13px;
  color: var(--danger);
  background: rgba(243, 139, 168, 0.1);
  padding: 10px 12px;
  border-radius: 6px;
  margin-bottom: 12px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

/* 表单 */
.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

.form-error {
  display: block;
  font-size: 12px;
  color: var(--danger);
  margin-top: 4px;
}
</style>
