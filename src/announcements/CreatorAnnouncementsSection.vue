<script setup lang="ts">
import HelpHint from '../components/HelpHint.vue'
import { ROLE_PACK_CREATOR_MESSAGE_FILENAME } from '../lib/rolePackCreatorMessage'
import { CREATOR_ANNOUNCE_HINT_PARAGRAPHS } from './helpCopy'

defineProps<{
  roleIds: string[]
  echoLines: string[]
  ocliveRolesDir: string
}>()

const launcherEchoRoleId = defineModel<string>('launcherEchoRoleId', { required: true })

const emit = defineEmits<{
  persistFollow: []
  refreshRoles: []
  clearFollow: []
}>()
</script>

<template>
  <section class="card announce-board announce-board--creator">
    <div class="section-title-row">
      <h2>创作者公告</h2>
      <HelpHint :paragraphs="[...CREATOR_ANNOUNCE_HINT_PARAGRAPHS]" />
    </div>

    <div class="creator-announce-bar" aria-label="选择要展示公告的角色">
      <label class="sr-only" for="launcher-echo-role">角色</label>
      <select
        id="launcher-echo-role"
        v-model="launcherEchoRoleId"
        class="modal-select creator-announce-select"
        :disabled="!ocliveRolesDir?.trim()"
        @change="emit('persistFollow')"
      >
        <option value="">选择角色…</option>
        <option v-for="id in roleIds" :key="id" :value="id">{{ id }}</option>
      </select>
      <button type="button" class="btn" :disabled="!ocliveRolesDir?.trim()" @click="emit('refreshRoles')">
        刷新列表
      </button>
      <button
        type="button"
        class="btn"
        :disabled="!launcherEchoRoleId?.trim()"
        title="清空当前选中的角色（不删包内文件）"
        @click="emit('clearFollow')"
      >
        取消跟随
      </button>
    </div>
    <p class="hint tiny creator-announce-bar-hint">
      「取消跟随」= 不再锁定要展示公告的角色；角色文件夹与
      <code>{{ ROLE_PACK_CREATOR_MESSAGE_FILENAME }}</code> 不会被删除。
    </p>

    <div class="creator-announce-panel">
      <div v-if="launcherEchoRoleId?.trim() && echoLines.length" class="creator-echo-wall">
        <p class="creator-echo-wall-label">「{{ launcherEchoRoleId }}」</p>
        <ul class="creator-echo-list">
          <li v-for="(line, idx) in echoLines" :key="idx" class="creator-echo-line">
            {{ line }}
          </li>
        </ul>
      </div>
      <p v-else-if="launcherEchoRoleId?.trim() && !echoLines.length" class="hint creator-echo-empty">
        该包内尚无 <code>{{ ROLE_PACK_CREATOR_MESSAGE_FILENAME }}</code>。
      </p>
      <p v-else class="hint creator-echo-empty">请选择角色。</p>
    </div>
  </section>
</template>

<style scoped>
.card {
  scroll-margin-top: 0.75rem;
  background: color-mix(in srgb, var(--fluent-bg-card) 82%, transparent);
  backdrop-filter: blur(9px) saturate(106%);
  -webkit-backdrop-filter: blur(9px) saturate(106%);
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius-lg);
  padding: 1rem 1.15rem;
  box-shadow: var(--fluent-shadow-card);
  transition:
    box-shadow 0.22s ease,
    border-color 0.22s ease;
}

.section-title-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.15rem;
  margin-bottom: 0.5rem;
}

.section-title-row h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.announce-board--creator {
  border-left: 3px solid var(--rail-accent-editor);
}

.hint {
  margin: 0 0 0.75rem;
  font-size: 0.8125rem;
  color: var(--fluent-text-secondary);
  line-height: 1.45;
}

.hint.tiny {
  margin: 0.35rem 0 0;
  font-size: 0.78rem;
  line-height: 1.5;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.creator-announce-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  align-items: center;
  margin-top: 0.5rem;
}

.creator-announce-select {
  flex: 1 1 14rem;
  min-width: 11rem;
}

.creator-announce-panel {
  margin-top: 0.65rem;
  padding: 0.75rem 0.9rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-stroke);
  background: var(--fluent-bg-subtle);
  min-height: 3.25rem;
}

.creator-announce-panel .creator-echo-wall {
  margin-top: 0;
}

.creator-announce-panel .creator-echo-empty {
  margin: 0;
}

.creator-echo-wall-label {
  margin: 0 0 0.5rem;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--fluent-text-secondary);
}

.creator-echo-list {
  display: flex;
  max-height: 14rem;
  flex-direction: column;
  gap: 0.55rem;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  list-style: none;
}

.creator-echo-line {
  margin: 0;
  padding: 0.5rem 0.65rem;
  border-left: 2px solid var(--rail-accent-editor);
  border-radius: var(--fluent-radius);
  font-size: 0.8125rem;
  font-style: italic;
  line-height: 1.45;
  color: var(--fluent-text-secondary);
  background: var(--fluent-bg-subtle);
}

.creator-echo-empty {
  margin: 0.5rem 0 0;
}
</style>
