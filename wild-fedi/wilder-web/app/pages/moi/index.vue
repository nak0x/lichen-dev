<script setup lang="ts">
// Personal space — profile for now; notes, research, the determination key and
// the "since last visit" feed arrive in later phases (links shown as upcoming).
definePageMeta({ layout: 'app', middleware: 'auth' })

const { user } = useAuth()

const roleLabel: Record<string, string> = {
  school: 'École · administration',
  student: 'Élève',
  service: 'Station de capture'
}

useHead({ title: 'Mon espace — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Mon espace</p>
    <h1>{{ user?.display_name || user?.username }}</h1>
    <p class="lede">Votre profil et votre espace personnel sur le réseau.</p>

    <dl class="dl">
      <div><dt>Identifiant</dt><dd class="mono">@{{ user?.username }}</dd></div>
      <div><dt>Rôle</dt><dd>{{ roleLabel[user?.role || ''] || user?.role }}</dd></div>
      <div v-if="user?.email"><dt>Courriel</dt><dd>{{ user.email }}</dd></div>
    </dl>

    <h2>Raccourcis</h2>
    <div class="soon">
      <NuxtLink class="card" to="/moi/feed"><h3>Mon fil →</h3><p>Ce qui a changé dans vos groupes depuis votre dernière visite.</p></NuxtLink>
      <NuxtLink class="card" to="/groupes"><h3>Groupes →</h3><p>Rejoindre un cercle d'étude ou en créer un.</p></NuxtLink>
      <article class="card soon-item"><h3>Notes personnelles</h3><p>À venir : vos notes privées.</p></article>
      <article class="card soon-item"><h3>Clé de détermination</h3><p>À venir : identifier une espèce du général au précis.</p></article>
    </div>
  </div>
</template>

<style scoped>
.soon { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr)); }
.soon h3 { margin: 0 0 .3rem; font-size: 1rem; }
.soon p { margin: 0; font-size: .9rem; color: var(--ink-soft); }
</style>
