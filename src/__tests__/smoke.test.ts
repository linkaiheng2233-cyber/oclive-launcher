import { describe, expect, it, vi } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import App from '../App.vue'

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })),
})

describe('launcher smoke', () => {
  it('mounts App without throwing', () => {
    const i18n = createI18n({
      legacy: false,
      locale: 'zh-CN',
      messages: { 'zh-CN': {}, 'en-US': {} },
    })
    const wrapper = shallowMount(App, {
      global: {
        plugins: [i18n],
      },
    })
    expect(wrapper.exists()).toBe(true)
  })
})
