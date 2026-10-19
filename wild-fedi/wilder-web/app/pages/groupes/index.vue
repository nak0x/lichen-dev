<script setup lang="ts">
// Browse and create groups. Any account may start one (classes and students).
definePageMeta({ middleware: 'auth' })

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
const showForm = ref(false)

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
    <div class="head">
      <div>
        <p class="oeil"><Fa icon="users" /> Groupes</p>
        <h1>Cercles d'étude</h1>
        <p class="lede">Une classe ou un sujet que l'on suit à plusieurs, et qui fédère.</p>
      </div>
      <button class="btn" @click="showForm = !showForm">
        <Fa :icon="showForm ? 'xmark' : 'plus'" /> {{ showForm ? 'Annuler' : 'Nouveau groupe' }}
      </button>
    </div>

    <Transition name="list">
      <form v-if="showForm" class="new card" @submit.prevent="create">
        <div class="row">
          <label class="field">Identifiant
            <input v-model="form.username" type="text" placeholder="castor" autocapitalize="off" spellcheck="false" required />
          </label>
          <label class="field">Nom
            <input v-model="form.name" type="text" placeholder="Classe Castor" required />
          </label>
        </div>
        <label class="field">Sujet d'étude
          <input v-model="form.subject" type="text" placeholder="la rivière et ses habitants" />
        </label>
        <div class="actions">
          <button class="btn" type="submit" :class="{ loading: busy }" :disabled="busy"><Fa icon="plus" /> Créer le groupe</button>
          <span v-if="error" class="err">{{ error }}</span>
        </div>
      </form>
    </Transition>

    <template v-if="mine && mine.length">
      <h2>Mes groupes</h2>
      <div class="glist stagger">
        <NuxtLink v-for="g in mine" :key="g.username" class="card gcard" :to="`/groupe/${g.username}`">
          <span class="gmark"><Fa icon="leaf" /></span>
          <span class="gbody">
            <strong>{{ g.name }}</strong>
            <span class="meta">{{ g.subject || 'sans sujet défini' }}</span>
          </span>
          <Fa icon="chevron-right" class="chev" />
        </NuxtLink>
      </div>
    </template>

    <h2>Tous les groupes</h2>
    <div v-if="groups && groups.length" class="glist stagger">
      <NuxtLink v-for="g in groups" :key="g.username" class="card gcard" :to="`/groupe/${g.username}`">
        <span class="gmark"><Fa icon="leaf" /></span>
        <span class="gbody">
          <strong>{{ g.name }}</strong>
          <span class="meta">@{{ g.username }} · {{ g.subject || 'sans sujet' }}</span>
        </span>
        <Fa icon="chevron-right" class="chev" />
      </NuxtLink>
    </div>
    <div v-else class="empty">
      <Fa icon="users" class="fa-icon" />
      <p>Aucun groupe pour l'instant. Créez le premier cercle d'étude.</p>
    </div>
  </div>
</template>

<style scoped>
.head { display: flex; align-items: flex-end; justify-content: space-between; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.2rem; }
.head h1 { margin: 0; }
.new { margin-bottom: 1.4rem; display: flex; flex-direction: column; gap: .9rem; }
.row { display: grid; gap: .9rem; grid-template-columns: 1fr 1fr; }
@media (max-width: 34rem) { .row { grid-template-columns: 1fr; } }
.glist { display: grid; gap: .7rem; grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr)); }
.gcard { display: flex; align-items: center; gap: .9rem; padding: 1rem 1.1rem; }
.gmark { width: 2.4rem; height: 2.4rem; flex: none; display: grid; place-items: center; border-radius: 40% 60% 58% 42% / 54% 44% 56% 46%; background: var(--sage-wash); color: var(--primary); }
.gbody { display: flex; flex-direction: column; min-width: 0; flex: 1; }
.gbody strong { font-family: var(--display); font-weight: 560; }
.chev { color: var(--sage); flex: none; }
</style>
