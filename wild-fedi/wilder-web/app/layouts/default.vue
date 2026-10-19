<script setup lang="ts">
// The single application shell for everyone (logged in or not): a persistent
// sidebar + the content column, with a mobile top bar that toggles the sidebar
// as an off-canvas drawer. The global Composer modal lives here so it can be
// opened from anywhere.
const drawer = useState('drawer-open', () => false)
const route = useRoute()
watch(() => route.fullPath, () => { drawer.value = false })
</script>

<template>
  <div class="app" :class="{ 'drawer-open': drawer }">
    <AppSidebar />
    <div class="sb-scrim" @click="drawer = false" />

    <div class="app-main">
      <div class="appbar">
        <button class="burger" aria-label="Menu" @click="drawer = !drawer"><Fa icon="bars" /></button>
        <NuxtLink to="/" class="sb-brand">
          <span class="mark" aria-hidden="true"><Fa icon="leaf" /></span> Wilder
        </NuxtLink>
      </div>
      <main>
        <slot />
      </main>
    </div>

    <Composer />
  </div>
</template>
