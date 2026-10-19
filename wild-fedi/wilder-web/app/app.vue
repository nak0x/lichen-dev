<script setup lang="ts">
// Apply the chosen theme to <html> on both server and client so first paint is
// already correct (no flash). `auto` leaves the attribute off, letting the
// prefers-color-scheme fallback in main.css decide.
const { theme } = useTheme()
useHead({
  htmlAttrs: {
    'data-theme': computed(() => (theme.value === 'auto' ? undefined : theme.value))
  },
  meta: [
    {
      name: 'theme-color',
      content: computed(() => (theme.value === 'dark' ? '#11160f' : '#e9eee2'))
    }
  ]
})
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
