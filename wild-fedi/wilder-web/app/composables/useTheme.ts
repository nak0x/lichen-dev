// Light / dark / auto theme, persisted in a cookie so the server can render the
// right `data-theme` on first paint (no flash). `auto` deliberately stores no
// attribute, letting the `prefers-color-scheme` fallback in main.css take over.
// Shared reactive state via useState; app.vue applies it to <html>.
export type Theme = 'auto' | 'light' | 'dark'

export function useTheme() {
  const cookie = useCookie<Theme>('wilder-theme', {
    default: () => 'auto',
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 365
  })
  const theme = useState<Theme>('wilder-theme', () => cookie.value || 'auto')

  function set(t: Theme) {
    theme.value = t
    cookie.value = t
  }
  // auto → light → dark → auto
  function cycle() {
    set(theme.value === 'auto' ? 'light' : theme.value === 'light' ? 'dark' : 'auto')
  }

  return { theme, set, cycle }
}
