<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

defineProps<{
  /** 点击问号后显示的短说明（大白话） */
  text: string
}>()

const open = ref(false)
const root = ref<HTMLElement | null>(null)

function toggle(e: Event) {
  e.stopPropagation()
  open.value = !open.value
}

function onDocClick(e: MouseEvent) {
  if (!open.value) return
  const el = root.value
  if (el && !el.contains(e.target as Node)) open.value = false
}

function onDocKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  document.addEventListener('keydown', onDocKeydown)
})
onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onDocKeydown)
})
</script>

<template>
  <span ref="root" class="help-hint">
    <button
      type="button"
      class="help-btn"
      :aria-expanded="open"
      aria-label="查看说明"
      @click="toggle"
    >
      ?
    </button>
    <div v-if="open" class="help-pop" role="tooltip">
      {{ text }}
    </div>
  </span>
</template>

<style scoped>
.help-hint {
  display: inline-flex;
  align-items: center;
  vertical-align: middle;
  margin-left: 0.3rem;
  position: relative;
}

.help-btn {
  width: 1.125rem;
  height: 1.125rem;
  border-radius: 50%;
  border: 1px solid var(--fluent-border-control);
  background: var(--fluent-bg-subtle);
  color: var(--fluent-text-secondary);
  font-size: 0.65rem;
  font-weight: 700;
  cursor: pointer;
  padding: 0;
  line-height: 1;
  flex-shrink: 0;
}

.help-btn:hover {
  border-color: var(--fluent-accent);
  color: var(--fluent-accent);
  background: var(--fluent-accent-subtle);
}

.help-pop {
  position: absolute;
  left: 0;
  top: calc(100% + 6px);
  z-index: 200;
  min-width: 11rem;
  max-width: min(22rem, calc(100vw - 3rem));
  padding: 0.5rem 0.65rem;
  font-size: 0.8125rem;
  font-weight: 400;
  line-height: 1.45;
  color: var(--fluent-text-primary);
  background: var(--fluent-bg-card);
  border: 1px solid var(--fluent-border-stroke);
  border-radius: var(--fluent-radius-lg);
  box-shadow: var(--fluent-shadow-card);
}
</style>
