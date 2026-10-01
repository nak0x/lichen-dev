<script setup lang="ts">
// Group admin: manage members. The server enforces admin-only on every mutation;
// here we also hide the page's controls from non-admins.
definePageMeta({ layout: 'app', middleware: 'auth' })

const route = useRoute()
const username = route.params.username as string
const authFetch = useRequestFetch()

const { data: group } = await useAsyncData(`group-admin-${username}`, () =>
  authFetch<any>(`/api/backend/groups/${username}`)
)
const { data: members, refresh } = await useAsyncData(`members-${username}`, () =>
  authFetch<any[]>(`/api/backend/groups/${username}/members`)
)

async function remove(accountId: string) {
  await $fetch(`/api/backend/groups/${username}/members/${accountId}`, { method: 'DELETE' })
  await refresh()
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
      <h2>Membres</h2>
      <div class="scroll">
        <table>
          <thead><tr><th>Membre</th><th>Rôle</th><th></th></tr></thead>
          <tbody>
            <tr v-for="m in members" :key="m.account_id">
              <td>{{ m.display_name || m.username }} <span class="mono">@{{ m.username }}</span></td>
              <td>{{ m.role }}</td>
              <td>
                <button v-if="m.role !== 'admin'" class="btn danger sm" @click="remove(m.account_id)">
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
