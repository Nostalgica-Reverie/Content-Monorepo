import { defineStore } from 'pinia'
import { ref } from 'vue'

export const ideThemes = ['Packwand Dark', 'Tangled Dark'] as const
export type IdeTheme = (typeof ideThemes)[number]

const storageKey = 'packwand.ide-theme'

function storedTheme(): IdeTheme {
  const value = window.localStorage.getItem(storageKey)
  return ideThemes.includes(value as IdeTheme) ? value as IdeTheme : 'Packwand Dark'
}

export const useThemeStore = defineStore('theme', () => {
  const current = ref<IdeTheme>(storedTheme())

  function setTheme(theme: IdeTheme) {
    current.value = theme
    window.localStorage.setItem(storageKey, theme)
  }

  return { current, setTheme, themes: ideThemes }
})
