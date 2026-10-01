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

    <h2>À venir dans cet espace</h2>
    <div class="soon">
      <article class="card"><h3>Notes personnelles</h3><p>Vos notes privées, visibles de vous seul.</p></article>
      <article class="card"><h3>Recherche personnelle</h3><p>Vos documents de recherche.</p></article>
      <article class="card"><h3>Clé de détermination</h3><p>Identifier une espèce du général au précis.</p></article>
      <article class="card"><h3>Mon fil</h3><p>Ce qui a changé depuis votre dernière visite.</p></article>
    </div>
  </div>
</template>

<style scoped>
.soon { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr)); }
.soon h3 { margin: 0 0 .3rem; font-size: 1rem; }
.soon p { margin: 0; font-size: .9rem; color: var(--ink-soft); }
</style>
