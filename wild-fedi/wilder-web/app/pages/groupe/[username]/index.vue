<script setup lang="ts">
// A group's space: public view with posts (filterable), a "captured" tab for
// capture-station content, a composer for members, join/leave, and — for admins
// — a link to the admin view.
definePageMeta({ layout: 'app', middleware: 'auth' })

const route = useRoute()
const username = route.params.username as string
const authFetch = useRequestFetch()

const { data: group, refresh: refreshGroup } = await useAsyncData(`group-${username}`, () =>
  authFetch<any>(`/api/backend/groups/${username}`)
)

const tab = ref<'posts' | 'captured'>('posts')
const tag = ref('')

const { data: posts, refresh: refreshPosts } = await useAsyncData(
  `posts-${username}`,
  () => {
    const q = tag.value ? `?tag=${encodeURIComponent(tag.value)}` : ''
    return authFetch<any[]>(`/api/backend/groups/${username}/posts${q}`)
  },
  { watch: [tag] }
)
const { data: captured, refresh: refreshCaptured } = await useAsyncData(`captured-${username}`, () =>
  authFetch<any[]>(`/api/backend/groups/${username}/captured`)
)

// Composer
const draft = reactive({ content: '', tags: '' })
const posting = ref(false)
async function publish() {
  posting.value = true
  try {
    await $fetch('/api/backend/posts', {
      method: 'POST',
      body: {
        group: username,
        content: draft.content,
        tags: draft.tags.split(/[\s,]+/).filter(Boolean)
      }
    })
    draft.content = ''
    draft.tags = ''
    await Promise.all([refreshPosts(), refreshCaptured()])
  } finally {
    posting.value = false
  }
}

async function join() {
  await $fetch(`/api/backend/groups/${username}/join`, { method: 'POST' })
  await refreshGroup()
}
async function leave() {
  await $fetch(`/api/backend/groups/${username}/leave`, { method: 'POST' })
  await refreshGroup()
}

useHead({ title: () => (group.value ? `${group.value.name} — Wilder` : 'Groupe') })
</script>

<template>
  <div class="wrap" v-if="group">
    <p class="oeil">Groupe · {{ group.subject || 'cercle d’étude' }}</p>
    <div class="ghead">
      <h1>{{ group.name }}</h1>
      <div class="actions">
        <NuxtLink v-if="group.is_admin" class="btn ghost sm" :to="`/groupe/${username}/admin`">Admin</NuxtLink>
        <button v-if="!group.is_member" class="btn sm" @click="join">Rejoindre</button>
        <button v-else-if="!group.is_admin" class="btn ghost sm" @click="leave">Quitter</button>
      </div>
    </div>
    <p v-if="group.summary" class="lede">{{ group.summary }}</p>

    <!-- Composer: members only -->
    <form v-if="group.is_member" class="composer card" @submit.prevent="publish">
      <textarea v-model="draft.content" rows="3" placeholder="Partager une observation…" required />
      <div class="composer-foot">
        <input v-model="draft.tags" type="text" placeholder="étiquettes : castor nuit" />
        <button class="btn" type="submit" :disabled="posting || !draft.content">{{ posting ? '…' : 'Publier' }}</button>
      </div>
    </form>

    <!-- Tabs -->
    <div class="tabs">
      <button :class="['tabbtn', { on: tab === 'posts' }]" @click="tab = 'posts'">Observations</button>
      <button :class="['tabbtn', { on: tab === 'captured' }]" @click="tab = 'captured'">Captations</button>
    </div>

    <template v-if="tab === 'posts'">
      <div class="filter">
        <input v-model="tag" type="search" placeholder="filtrer par étiquette (sans #)" />
        <button v-if="tag" class="btn ghost sm" @click="tag = ''">Effacer</button>
      </div>
      <div v-if="posts && posts.length" class="feed">
        <PostItem v-for="p in posts" :key="p.id" :post="p" />
      </div>
      <div v-else class="empty">Aucune observation{{ tag ? ' pour cette étiquette' : '' }}.</div>
    </template>

    <template v-else>
      <p class="note">
        <strong>Captations.</strong> Uniquement les publications des stations de capture
        (comptes de type <code>Service</code>) de ce groupe.
      </p>
      <div v-if="captured && captured.length" class="feed">
        <PostItem v-for="p in captured" :key="p.id" :post="p" />
      </div>
      <div v-else class="empty">Aucune captation.</div>
    </template>
  </div>
</template>

<style scoped>
.ghead { display: flex; align-items: baseline; justify-content: space-between; gap: 1rem; flex-wrap: wrap; }
.ghead h1 { margin: 0; }
.composer { margin: 1.2rem 0; }
.composer textarea { width: 100%; }
.composer-foot { display: flex; gap: .5rem; margin-top: .5rem; align-items: center; }
.composer-foot input { flex: 1; }
.composer-foot .btn { flex: none; }
.tabs { display: flex; gap: .3rem; border-bottom: 1px solid var(--line-strong); margin: 1.4rem 0 1rem; }
.tabbtn { font: inherit; font-size: .9rem; background: none; border: 0; padding: .5rem .8rem; cursor: pointer; color: var(--ink-soft); border-bottom: 2px solid transparent; margin-bottom: -1px; }
.tabbtn.on { color: var(--forest); border-bottom-color: var(--forest); font-weight: 600; }
.filter { display: flex; gap: .5rem; margin-bottom: 1rem; }
.filter input { flex: 1; }
</style>
