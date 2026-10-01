<script setup lang="ts">
// Browse and create groups. Any account may start one (classes and students).
definePageMeta({ layout: 'app', middleware: 'auth' })

const authFetch = useRequestFetch()
const { data: groups } = await useAsyncData('groups-all', () =>
  authFetch<any[]>('/api/backend/groups')
)
const { data: mine } = await useAsyncData('groups-mine', () =>
  authFetch<any[]>('/api/backend/groups/mine')
)

const router = useRouter()
const form = reactive({ username: '', name: '', subject: '' })
const error = ref('')
const busy = ref(false)

async function create() {
  error.value = ''
  busy.value = true
  try {
    const g = await $fetch<any>('/api/backend/groups', {
      method: 'POST',
      body: { username: form.username, name: form.name, subject: form.subject || undefined }
    })
    router.push(`/groupe/${g.username}`)
  } catch (e: any) {
    error.value = e?.data?.error || 'Création impossible.'
  } finally {
    busy.value = false
  }
}

useHead({ title: 'Groupes — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Groupes</p>
    <h1>Cercles d'étude</h1>
    <p class="lede">Une classe ou un sujet que l'on suit à plusieurs, et qui fédère.</p>

    <h2>Nouveau groupe</h2>
    <form class="new" @submit.prevent="create">
      <input v-model="form.username" type="text" placeholder="identifiant (castor)" autocapitalize="off" spellcheck="false" required />
      <input v-model="form.name" type="text" placeholder="nom (Classe Castor)" required />
      <input v-model="form.subject" type="text" placeholder="sujet d'étude" />
      <button class="btn" type="submit" :disabled="busy">{{ busy ? '…' : 'Créer' }}</button>
    </form>
    <p v-if="error" class="err">{{ error }}</p>

    <template v-if="mine && mine.length">
      <h2>Mes groupes</h2>
      <ul class="glist">
        <li v-for="g in mine" :key="g.username">
          <NuxtLink :to="`/groupe/${g.username}`">{{ g.name }}</NuxtLink>
          <span class="meta">· {{ g.subject || 'sans sujet' }}</span>
        </li>
      </ul>
    </template>

    <h2>Tous les groupes</h2>
    <ul v-if="groups && groups.length" class="glist">
      <li v-for="g in groups" :key="g.username">
        <NuxtLink :to="`/groupe/${g.username}`">{{ g.name }}</NuxtLink>
        <span class="meta">@{{ g.username }} · {{ g.subject || 'sans sujet' }}</span>
      </li>
    </ul>
    <div v-else class="empty">Aucun groupe pour l'instant.</div>
  </div>
</template>

<style scoped>
.new { display: flex; gap: .5rem; flex-wrap: wrap; align-items: center; }
.new input { flex: 1 1 10rem; }
.new .btn { flex: none; }
.glist { list-style: none; padding: 0; }
.glist li { padding: .55rem 0; border-top: 1px solid var(--line); }
.glist a { font-weight: 600; }
.glist .meta { margin-left: .5rem; }
</style>
