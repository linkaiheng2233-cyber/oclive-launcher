<script setup lang="ts">
import HelpHint from '../components/HelpHint.vue'
import { DEVELOPER_ANNOUNCE_HINT_PARAGRAPHS } from './helpCopy'

defineProps<{
  announceEditEnabled: boolean
  fetchBusy: boolean
}>()

const url = defineModel<string>('url', { required: true })
const body = defineModel<string>('body', { required: true })

const emit = defineEmits<{
  save: []
  fetch: []
}>()
</script>

<template>
  <section class="card announce-board announce-board--developer">
    <div class="section-title-row">
      <h2>开发者公告</h2>
      <HelpHint :paragraphs="[...DEVELOPER_ANNOUNCE_HINT_PARAGRAPHS]" />
    </div>
    <p class="hint tiny">只读；维护者构建可编辑保存。</p>
    <div class="dev-announce-url-row">
      <label class="modal-label" for="dev-announce-url">远程正文地址（可选）</label>
      <input
        id="dev-announce-url"
        v-model="url"
        type="url"
        class="dev-announce-url-input"
        placeholder="https://raw.githubusercontent.com/你的用户/你的仓库/main/公告.md"
      />
      <button type="button" class="btn" :disabled="fetchBusy || !url?.trim()" @click="emit('fetch')">
        {{ fetchBusy ? '拉取中…' : '拉取最新' }}
      </button>
    </div>
    <p class="hint tiny">
      不是复制仓库主页链接：需要 Raw 或等价直链（响应体就是 Markdown/纯文本）。改 URL 后请先「保存配置」，再拉取。
    </p>
    <template v-if="announceEditEnabled">
      <textarea v-model="body" class="announce-edit" rows="7" spellcheck="false" />
      <button type="button" class="btn" @click="emit('save')">保存到本地</button>
    </template>
    <template v-else>
      <p v-if="!body.trim()" class="hint announce-empty-inline">暂无开发者公告。</p>
      <pre v-else class="announce-readonly">{{ body }}</pre>
    </template>
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

.announce-board--developer {
  border-left: 3px solid var(--fluent-accent);
}

.hint {
  margin: 0 0 0.75rem;
  font-size: 0.8125rem;
  color: var(--fluent-text-secondary);
  line-height: 1.45;
}

.hint.tiny {
  margin: 0 0 0.45rem;
  font-size: 0.78rem;
  line-height: 1.5;
}

.modal-label {
  display: block;
  width: 100%;
  font-size: 0.75rem;
  font-weight: 600;
  margin-top: 0.35rem;
  margin-bottom: 0.25rem;
  color: var(--fluent-text-secondary);
}

.dev-announce-url-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  margin-top: 0.35rem;
}

.dev-announce-url-input {
  flex: 1 1 14rem;
  min-width: 10rem;
  padding: 0.4rem 0.55rem;
  font: inherit;
  font-size: 0.8125rem;
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius);
  background: var(--fluent-bg-input);
  color: var(--fluent-text-primary);
}

.announce-empty-inline {
  margin: 0.5rem 0 0;
  font-style: italic;
  color: var(--fluent-text-secondary);
}

.announce-readonly {
  margin: 0;
  padding: 0.65rem 0.75rem;
  font-family: var(--fluent-font);
  font-size: 0.8125rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  background: var(--fluent-bg-subtle);
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius);
  color: var(--fluent-text-primary);
}

.announce-edit {
  width: 100%;
  box-sizing: border-box;
  font-family: var(--fluent-mono);
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  border-radius: var(--fluent-radius);
  border: 1px solid var(--fluent-border-control);
  padding: 0.5rem 0.65rem;
  background: var(--fluent-bg-input);
  color: var(--fluent-text-primary);
}

.announce-edit:focus {
  outline: none;
  border-color: var(--fluent-border-focus);
  box-shadow: 0 0 0 1px var(--fluent-border-focus);
}
</style>
