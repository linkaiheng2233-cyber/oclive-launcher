import { computed, onMounted, ref, watch } from 'vue'

/** 与 oclive-pack-editor `usePackShellPreferences` 中档位一致，便于习惯对齐 */
const SCALE_STORAGE_KEY = 'oclive-launcher-ui-scale'
const UI_SCALE_STEPS = [0.8, 0.88, 0.96, 1, 1.08, 1.16, 1.24] as const

function nearestScaleIndex(value: number): number {
  let best = 3
  let bestDiff = Infinity
  UI_SCALE_STEPS.forEach((s, i) => {
    const d = Math.abs(s - value)
    if (d < bestDiff) {
      bestDiff = d
      best = i
    }
  })
  return best
}

function readStoredScaleIndex(): number {
  try {
    const raw = localStorage.getItem(SCALE_STORAGE_KEY)
    if (raw == null || raw === '') return 3
    const n = Number(raw)
    if (!Number.isFinite(n)) return 3
    return nearestScaleIndex(n)
  } catch {
    return 3
  }
}

export function useLauncherUiScale() {
  const uiScaleIndex = ref(3)

  function applyScaleToDocument() {
    const scale = UI_SCALE_STEPS[uiScaleIndex.value] ?? 1
    document.documentElement.style.setProperty('--launcher-ui-scale', String(scale))
  }

  onMounted(() => {
    uiScaleIndex.value = readStoredScaleIndex()
    applyScaleToDocument()
  })

  watch(uiScaleIndex, (i) => {
    const scale = UI_SCALE_STEPS[i] ?? 1
    try {
      localStorage.setItem(SCALE_STORAGE_KEY, String(scale))
    } catch {
      /* ignore */
    }
    applyScaleToDocument()
  })

  function bumpScale(delta: number) {
    uiScaleIndex.value = Math.max(0, Math.min(UI_SCALE_STEPS.length - 1, uiScaleIndex.value + delta))
  }

  const scaleLabel = computed(() => `${Math.round((UI_SCALE_STEPS[uiScaleIndex.value] ?? 1) * 100)}%`)

  return { bumpScale, scaleLabel }
}
