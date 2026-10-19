<script setup lang="ts">
// The one navigation for everyone. Two sections — Personnel and Fédération —
// plus a school-only École group. Logged-out, Personnel becomes a warm sign-in
// invitation; Fédération stays fully browsable, so both states feel like one
// product. Collapses to an icon rail (<=64rem) then an off-canvas drawer (<48rem).
const { user, isSchool, logout } = useAuth()
const { openComposer } = useComposer()
const { theme, set } = useTheme()
const router = useRouter()

const initial = computed(() =>
  (user.value?.display_name || user.value?.username || '?').charAt(0).toUpperCase()
)
const roleLabel: Record<string, string> = {
  school: 'École', student: 'Élève', service: 'Station'
}

async function onLogout() {
  await logout()
  router.push('/')
}
</script>

<template>
  <aside class="sidebar">
    <NuxtLink to="/" class="sb-brand">
      <span class="mark" aria-hidden="true"><Fa icon="leaf" /></span>
      <span class="label">Wilder<small>carnet de terrain</small></span>
    </NuxtLink>

    <button v-if="user" class="btn accent sb-publish" @click="openComposer()">
      <Fa icon="feather" /> <span class="label">Publier</span>
    </button>

    <!-- Personnel -->
    <template v-if="user">
      <nav class="sb-section" aria-label="Personnel">
        <p class="sb-section-label">Personnel</p>
        <NuxtLink class="sb-link" to="/moi/feed"><Fa icon="house" fixed-width /><span class="label">Mon fil</span></NuxtLink>
        <NuxtLink class="sb-link" to="/moi"><Fa icon="user" fixed-width /><span class="label">Mon profil</span></NuxtLink>
        <NuxtLink class="sb-link" to="/moi/notes"><Fa icon="book-open" fixed-width /><span class="label">Mes notes</span></NuxtLink>
        <NuxtLink class="sb-link" to="/moi/recherche"><Fa icon="flask" fixed-width /><span class="label">Ma recherche</span></NuxtLink>
        <NuxtLink class="sb-link" to="/determiner"><Fa icon="key" fixed-width /><span class="label">Déterminer</span></NuxtLink>
        <NuxtLink class="sb-link" to="/groupes"><Fa icon="users" fixed-width /><span class="label">Mes groupes</span></NuxtLink>
      </nav>
      <nav v-if="isSchool" class="sb-section" aria-label="École">
        <p class="sb-section-label">École</p>
        <NuxtLink class="sb-link" to="/ecole"><Fa icon="school" fixed-width /><span class="label">Administration</span></NuxtLink>
      </nav>
    </template>

    <div v-else class="sb-section signin">
      <p class="sb-section-label label">Rejoindre</p>
      <NuxtLink class="btn sb-publish" to="/login"><Fa icon="right-to-bracket" /> <span class="label">Se connecter</span></NuxtLink>
      <p class="label hint">Les élèves et écoles publient et suivent leurs observations ici.</p>
    </div>

    <!-- Fédération (tout le monde) -->
    <nav class="sb-section" aria-label="Fédération">
      <p class="sb-section-label">Fédération</p>
      <NuxtLink class="sb-link" to="/"><Fa icon="globe" fixed-width /><span class="label">Activité</span></NuxtLink>
      <NuxtLink class="sb-link" to="/carte"><Fa icon="map" fixed-width /><span class="label">Carte</span></NuxtLink>
      <NuxtLink class="sb-link" to="/acteur"><Fa icon="binoculars" fixed-width /><span class="label">Explorer</span></NuxtLink>
      <NuxtLink class="sb-link" to="/actualites"><Fa icon="bell" fixed-width /><span class="label">Actualités</span></NuxtLink>
      <NuxtLink class="sb-link" to="/instance"><Fa icon="chart-line" fixed-width /><span class="label">Instance</span></NuxtLink>
      <NuxtLink class="sb-link" to="/a-propos"><Fa icon="circle-info" fixed-width /><span class="label">À propos</span></NuxtLink>
    </nav>

    <div class="sb-foot">
      <div class="theme-toggle" role="group" aria-label="Thème">
        <button :class="{ on: theme === 'auto' }" aria-label="Thème système" title="Système" @click="set('auto')"><Fa icon="circle-half-stroke" /></button>
        <button :class="{ on: theme === 'light' }" aria-label="Thème clair" title="Clair" @click="set('light')"><Fa icon="sun" /></button>
        <button :class="{ on: theme === 'dark' }" aria-label="Thème sombre" title="Sombre" @click="set('dark')"><Fa icon="moon" /></button>
      </div>

      <template v-if="user">
        <NuxtLink to="/moi" class="sb-account">
          <span class="sb-avatar">{{ initial }}</span>
          <span class="meta-col label">
            <span class="who">{{ user.display_name || user.username }}</span>
            <span class="role">{{ roleLabel[user.role] || user.role }}</span>
          </span>
        </NuxtLink>
        <button class="sb-link logout" @click="onLogout"><Fa icon="right-from-bracket" fixed-width /><span class="label">Déconnexion</span></button>
      </template>
    </div>
  </aside>
</template>

<style scoped>
.signin { margin-top: .4rem; }
.signin .sb-publish { width: calc(100% - .4rem); margin: .2rem .2rem; }
.signin .hint { font-size: .78rem; color: var(--ink-soft); padding: .2rem .55rem; margin: .2rem 0 0; }
.logout { width: 100%; background: none; border: 0; font: inherit; cursor: pointer; text-align: left; }
.sb-foot .theme-toggle { align-self: flex-start; }
</style>
