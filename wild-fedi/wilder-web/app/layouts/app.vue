<script setup lang="ts">
// Shell for the authenticated app. The public federation browser keeps using
// `default.vue`; signed-in pages use this one.
const { user, isSchool, logout } = useAuth()
const router = useRouter()
const route = useRoute()
const annee = new Date().getFullYear()

// Mobile menu open state — collapses automatically whenever the route changes
// so a tap-through never leaves the panel hanging open.
const menuOpen = ref(false)
watch(() => route.fullPath, () => { menuOpen.value = false })

async function onLogout() {
  await logout()
  router.push('/login')
}
</script>

<template>
  <div>
    <header class="top">
      <div class="wrap-wide top-in">
        <NuxtLink to="/moi" class="brand">Wilder <span>· espace</span></NuxtLink>

        <button
          class="nav-toggle"
          :aria-expanded="menuOpen"
          aria-controls="app-nav"
          aria-label="Menu"
          @click="menuOpen = !menuOpen"
        >
          <span aria-hidden="true">{{ menuOpen ? '✕' : '☰' }}</span> Menu
        </button>

        <nav id="app-nav" class="nav" :class="{ open: menuOpen }">
          <NuxtLink to="/moi">Mon espace</NuxtLink>
          <NuxtLink to="/moi/feed">Mon fil</NuxtLink>
          <NuxtLink to="/determiner">Déterminer</NuxtLink>
          <NuxtLink to="/groupes">Groupes</NuxtLink>
          <NuxtLink to="/actualites">Actualités</NuxtLink>
          <NuxtLink to="/">Fil fédéré</NuxtLink>
          <NuxtLink v-if="isSchool" to="/ecole">École</NuxtLink>
          <span class="user">
            <span v-if="user" class="who">{{ user.display_name || user.username }}</span>
            <a href="#" @click.prevent="onLogout">Déconnexion</a>
          </span>
        </nav>
      </div>
    </header>

    <main>
      <slot />
    </main>

    <footer class="foot">
      <div class="wrap-wide">
        <p>
          Espace du réseau LICHEN · Données hébergées dans l'Union européenne ·
          Aucun traceur, aucun cookie tiers.
        </p>
        <p>© {{ annee }} Projet LICHEN · Contenu sous licence CC BY 4.0.</p>
      </div>
    </footer>
  </div>
</template>
