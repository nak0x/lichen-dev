<script setup lang="ts">
// School administration: instance figures, account provisioning (students and
// capture stations), and publishing events and news. Group creation lives on
// /groupes; member management on each group's admin page.
definePageMeta({ layout: 'app', middleware: ['auth', 'school'] })

import type { Account } from '~/composables/useAuth'

const authFetch = useRequestFetch()
const { data: stats, refresh: refreshStats } = await useAsyncData('stats', () =>
  authFetch<any>('/api/backend/instance/stats')
)
const { data: roster, refresh: refreshRoster } = await useAsyncData('roster', () =>
  authFetch<Account[]>('/api/backend/accounts')
)

const error = ref('')

// --- account provisioning (student or capture station) ---
const acct = reactive({ username: '', password: '', display_name: '', kind: 'students' })
const creatingAcct = ref(false)
async function createAccount() {
  error.value = ''
  creatingAcct.value = true
  try {
    await $fetch(`/api/backend/accounts/${acct.kind}`, {
      method: 'POST',
      body: {
        username: acct.username,
        password: acct.password,
        display_name: acct.display_name || undefined
      }
    })
    acct.username = ''
    acct.password = ''
    acct.display_name = ''
    await Promise.all([refreshRoster(), refreshStats()])
  } catch (e: any) {
    error.value = e?.data?.error || 'Création impossible.'
  } finally {
    creatingAcct.value = false
  }
}

// --- publishing events & news ---
const ev = reactive({ title: '', content: '', start_time: '', location: '' })
const nw = reactive({ title: '', content: '' })
const pub = ref(false)

async function publishEvent() {
  pub.value = true
  try {
    await $fetch('/api/backend/events', {
      method: 'POST',
      body: {
        title: ev.title,
        content: ev.content,
        start_time: ev.start_time ? new Date(ev.start_time).toISOString() : undefined,
        location: ev.location || undefined
      }
    })
    ev.title = ''; ev.content = ''; ev.start_time = ''; ev.location = ''
    await refreshStats()
  } finally {
    pub.value = false
  }
}
async function publishNews() {
  pub.value = true
  try {
    await $fetch('/api/backend/news', { method: 'POST', body: { title: nw.title, content: nw.content } })
    nw.title = ''; nw.content = ''
    await refreshStats()
  } finally {
    pub.value = false
  }
}

useHead({ title: 'École — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Administration</p>
    <h1>L'école</h1>
    <p class="lede">Gérez l'instance : comptes, groupes, événements et actualités.</p>

    <div v-if="stats" class="kpis">
      <div class="kpi"><b>{{ stats.students }}</b><span>élèves &amp; stations</span></div>
      <div class="kpi"><b>{{ stats.groups }}</b><span>groupes</span></div>
      <div class="kpi"><b>{{ stats.events }}</b><span>événements</span></div>
      <div class="kpi"><b>{{ stats.news }}</b><span>actualités</span></div>
    </div>

    <h2>Nouveau compte</h2>
    <form class="new" @submit.prevent="createAccount">
      <select v-model="acct.kind" aria-label="type de compte">
        <option value="students">Élève</option>
        <option value="stations">Station de capture</option>
      </select>
      <input v-model="acct.username" type="text" placeholder="identifiant" autocapitalize="off" spellcheck="false" required />
      <input v-model="acct.display_name" type="text" placeholder="nom affiché" />
      <input v-model="acct.password" type="password" placeholder="mot de passe (8+)" required />
      <button class="btn" type="submit" :disabled="creatingAcct">{{ creatingAcct ? '…' : 'Créer' }}</button>
    </form>
    <p v-if="error" class="err">{{ error }}</p>

    <div class="cols">
      <section>
        <h2>Publier un événement</h2>
        <form class="stack" @submit.prevent="publishEvent">
          <input v-model="ev.title" type="text" placeholder="titre" required />
          <textarea v-model="ev.content" rows="2" placeholder="description" required />
          <input v-model="ev.start_time" type="datetime-local" />
          <input v-model="ev.location" type="text" placeholder="lieu" />
          <button class="btn sm" type="submit" :disabled="pub">Publier l'événement</button>
        </form>
      </section>

      <section>
        <h2>Publier une actualité</h2>
        <form class="stack" @submit.prevent="publishNews">
          <input v-model="nw.title" type="text" placeholder="titre" required />
          <textarea v-model="nw.content" rows="4" placeholder="texte" required />
          <button class="btn sm" type="submit" :disabled="pub">Publier l'actualité</button>
        </form>
      </section>
    </div>

    <h2>Comptes créés</h2>
    <div v-if="roster && roster.length" class="scroll">
      <table>
        <thead><tr><th>Identifiant</th><th>Nom</th><th>Rôle</th><th>Créé le</th></tr></thead>
        <tbody>
          <tr v-for="a in roster" :key="a.id">
            <td class="mono">@{{ a.username }}</td>
            <td>{{ a.display_name || '—' }}</td>
            <td>{{ a.role }}</td>
            <td>{{ new Date(a.created_at).toLocaleDateString('fr-FR') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-else class="empty">Aucun compte créé pour l'instant.</div>
  </div>
</template>

<style scoped>
.kpis { margin: 1rem 0 1.6rem; }
.new { display: flex; gap: .5rem; flex-wrap: wrap; align-items: center; margin-bottom: .5rem; }
.new input { flex: 1 1 9rem; }
.new select, .new .btn { flex: none; }
.cols { display: grid; gap: 1.5rem; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); margin: .5rem 0 1rem; }
.stack { display: flex; flex-direction: column; gap: .5rem; }
.stack input, .stack textarea { width: 100%; }
.stack .btn { align-self: flex-start; }
</style>
