<script setup lang="ts">
// Group admin: manage members' trust tiers and moderate the review queue. The
// server enforces admin-only on every mutation; here we also hide the controls
// from non-admins. Trust tiers: guest (submissions reviewed), contributor
// (posts directly), admin (moderates).
definePageMeta({ middleware: 'auth' })

const route = useRoute()
const username = route.params.username as string
const authFetch = useRequestFetch()
const { user } = useAuth()

const ROLES = ['guest', 'contributor', 'admin'] as const

const { data: group } = await useAsyncData(`group-admin-${username}`, () =>
  authFetch<any>(`/api/backend/groups/${username}`)
)
const { data: members, refresh } = await useAsyncData(`members-${username}`, () =>
  authFetch<any[]>(`/api/backend/groups/${username}/members`)
)
const { data: pending, refresh: refreshPending } = await useAsyncData(`pending-${username}`, () =>
  authFetch<any[]>(`/api/backend/groups/${username}/pending`)
)

// Per-item in-flight tracking, so only the button being pressed spins.
const busy = reactive(new Set<string>())
const isBusy = (id: string) => busy.has(id)

async function setRole(accountId: string, role: string) {
  busy.add(accountId)
  try {
    await $fetch(`/api/backend/groups/${username}/members/${accountId}`, {
      method: 'PATCH',
      body: { role }
    })
    await refresh()
  } finally {
    busy.delete(accountId)
  }
}

async function remove(accountId: string) {
  busy.add(accountId)
  try {
    await $fetch(`/api/backend/groups/${username}/members/${accountId}`, { method: 'DELETE' })
    await refresh()
  } finally {
    busy.delete(accountId)
  }
}

async function decide(id: string, verb: 'approve' | 'reject') {
  busy.add(id)
  try {
    await $fetch(`/api/backend/posts/${id}/${verb}`, { method: 'POST' })
    await refreshPending()
  } finally {
    busy.delete(id)
  }
}

useHead({ title: `Admin — ${username}` })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Administration du groupe</p>
    <h1>{{ group?.name }}</h1>
    <p class="lede">
      <NuxtLink :to="`/groupe/${username}`">← retour au groupe</NuxtLink>
    </p>

    <div v-if="group && !group.is_admin" class="err">Réservé aux administrateurs du groupe.</div>

    <template v-else>
      <!-- Review queue: guest submissions awaiting a decision -->
      <h2>À valider <span v-if="pending && pending.length" class="badge">{{ pending.length }}</span></h2>
      <p class="hint">
        Les observations proposées par les invités attendent votre validation avant
        d’être publiées et diffusées.
      </p>
      <TransitionGroup v-if="pending && pending.length" name="list" tag="div" class="queue">
        <article v-for="p in pending" :key="p.id" class="card pend">
          <header>
            <strong>@{{ p.author }}</strong>
            <time>{{ new Date(p.published).toLocaleString('fr-FR') }}</time>
          </header>
          <p class="body">{{ p.content }}</p>
          <p v-if="p.tags && p.tags.length" class="tags">
            <span v-for="t in p.tags" :key="t" class="tag">{{ t }}</span>
          </p>
          <div class="pend-actions">
            <button class="btn sm" :class="{ loading: isBusy(p.id) }" :disabled="isBusy(p.id)" @click="decide(p.id, 'approve')">Valider</button>
            <button class="btn ghost danger sm" :disabled="isBusy(p.id)" @click="decide(p.id, 'reject')">Refuser</button>
          </div>
        </article>
      </TransitionGroup>
      <div v-else class="empty">Rien à valider.</div>

      <!-- Members: trust tiers -->
      <h2>Membres</h2>
      <div class="scroll">
        <table>
          <thead><tr><th>Membre</th><th>Rôle</th><th></th></tr></thead>
          <tbody>
            <tr v-for="m in members" :key="m.account_id" :class="{ dim: isBusy(m.account_id) }">
              <td>{{ m.display_name || m.username }} <span class="mono">@{{ m.username }}</span></td>
              <td>
                <span v-if="m.account_id === user?.id" class="self">{{ m.role }} (vous)</span>
                <select
                  v-else
                  :value="m.role"
                  :disabled="isBusy(m.account_id)"
                  @change="setRole(m.account_id, ($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="r in ROLES" :key="r" :value="r">{{ r }}</option>
                </select>
              </td>
              <td>
                <button
                  v-if="m.account_id !== user?.id"
                  class="btn ghost danger sm"
                  :disabled="isBusy(m.account_id)"
                  @click="remove(m.account_id)"
                >
                  Retirer
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </div>
</template>

<style scoped>
.badge { font-size: .7rem; background: var(--forest); color: #fff; border-radius: 1rem; padding: .05rem .5rem; vertical-align: middle; }
.hint { font-size: .85rem; color: var(--ink-soft); margin: .2rem 0 1rem; }
.queue { display: grid; gap: .8rem; margin-bottom: 1.6rem; }
.pend header { display: flex; justify-content: space-between; align-items: baseline; gap: 1rem; }
.pend time { font-size: .75rem; color: var(--ink-soft); }
.pend .body { margin: .5rem 0; white-space: pre-wrap; }
.pend-actions { display: flex; gap: .5rem; }
.tags { display: flex; gap: .3rem; flex-wrap: wrap; margin: .3rem 0; }
.tag { font-size: .72rem; color: var(--forest); }
/* Compact the role picker to sit neatly inside a table cell. */
td select { width: auto; min-width: 9rem; padding: .3rem .5rem; font-size: .85rem; }
.self { color: var(--ink-soft); font-size: .85rem; }
.mono { color: var(--ink-soft); font-size: .8rem; }
</style>
