// Global state for the observation composer so the sidebar "Publier" button and
// any empty-state call-to-action can open the same modal from anywhere.
// `posted` is bumped on every successful post; feed pages watch it to refresh.
export function useComposer() {
  const open = useState('composer-open', () => false)
  const presetGroup = useState<string | null>('composer-group', () => null)
  const posted = useState('composer-posted', () => 0)

  function openComposer(group?: string | null) {
    presetGroup.value = group ?? null
    open.value = true
  }
  function closeComposer() {
    open.value = false
  }
  return { open, presetGroup, posted, openComposer, closeComposer }
}
