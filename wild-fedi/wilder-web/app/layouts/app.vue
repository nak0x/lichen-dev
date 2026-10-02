<script setup lang="ts">
// Shell for the authenticated app. The public federation browser keeps using
// `default.vue`; signed-in pages use this one.
const { user, isSchool, logout } = useAuth()
const router = useRouter()
const annee = new Date().getFullYear()

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
        <nav>
          <NuxtLink to="/moi">Mon espace</NuxtLink>
          <NuxtLink to="/moi/feed">Mon fil</NuxtLink>
          <NuxtLink to="/determiner">Déterminer</NuxtLink>
          <NuxtLink to="/groupes">Groupes</NuxtLink>
          <NuxtLink to="/actualites">Actualités</NuxtLink>
          <NuxtLink to="/">Fil fédéré</NuxtLink>
          <NuxtLink v-if="isSchool" to="/ecole">École</NuxtLink>
          <span v-if="user" class="who">{{ user.display_name || user.username }}</span>
          <a href="#" @click.prevent="onLogout">Déconnexion</a>
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

<style scoped>
.who { color: var(--ink); font-weight: 500; }
</style>
