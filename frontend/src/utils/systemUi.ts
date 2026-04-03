export type SystemThemeMode = 'light' | 'dark'

function ensureMeta(name: string) {
  let meta = document.querySelector(`meta[name="${name}"]`) as HTMLMetaElement | null
  if (!meta) {
    meta = document.createElement('meta')
    meta.setAttribute('name', name)
    document.head.appendChild(meta)
  }
  return meta
}

export function applySystemTheme(mode: SystemThemeMode, themeColor?: string) {
  if (typeof document === 'undefined') return

  const resolvedThemeColor = themeColor || (mode === 'dark' ? '#141414' : '#faf9f7')

  document.documentElement.setAttribute('data-theme', mode)
  document.documentElement.style.setProperty('color-scheme', mode)
  document.documentElement.style.backgroundColor = resolvedThemeColor
  document.body.style.backgroundColor = resolvedThemeColor

  const themeMeta = ensureMeta('theme-color')
  themeMeta.setAttribute('content', resolvedThemeColor)

  const appleStatusMeta = ensureMeta('apple-mobile-web-app-status-bar-style')
  appleStatusMeta.setAttribute('content', 'black-translucent')
}
