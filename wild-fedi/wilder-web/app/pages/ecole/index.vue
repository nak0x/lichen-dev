<script setup lang="ts">
// School administration (Phase 1 slice): provision student accounts and see the
// roster. Events, news and group creation join this dashboard in later phases.
definePageMeta({ layout: 'app', middleware: ['auth', 'school'] })

import type { Account } from '~/composables/useAuth'

// Capture the request-scoped fetch in setup so SSR forwards the session cookie.
const authFetch = useRequestFetch()
const { data: roster, refresh } = await useAsyncData('roster', () =>
  authFetch<Account[]>('/api/backend/accounts')
)

const form = reactive({ username: '', password: '', display_name: '' })
const error = ref('')
const busy = ref(false)

async function createStudent() {
  error.value = ''
  busy.value = true
  try {
    await $fetch('/api/backend/accounts/students', {
      method: 'POST',
      body: {
        username: form.username,
        password: form.password,
        display_name: form.display_name || undefined
      }
    })
    form.username = ''
    form.password = ''
    form.display_name = ''
    await refresh()
  } catch (e: any) {
    error.value = e?.data?.error || 'Création impossible.'
  } finally {
    busy.value = false
  }
}

useHead({ title: 'École — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Administration</p>
    <h1>L'école</h1>
    <p class="lede">Créez les comptes élèves et gérez le réseau de l'instance.</p>

    <h2>Nouvel élève</h2>
    <form class="new" @submit.prevent="createStudent">
      <input v-model="form.username" type="text" placeholder="identifiant" autocapitalize="off" spellcheck="false" required />
      <input v-model="form.display_name" type="text" placeholder="nom affiché" />
      <input v-model="form.password" type="password" placeholder="mot de passe (8+)" required />
      <button class="btn" type="submit" :disabled="busy">{{ busy ? '…' : 'Créer' }}</button>
    </form>
    <p v-if="error" class="err">{{ error }}</p>

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
.new { display: flex; gap: .5rem; flex-wrap: wrap; align-items: center; margin-bottom: .5rem; }
.new input { flex: 1 1 10rem; }
.new .btn { flex: none; }
</style>
