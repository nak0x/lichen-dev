<script setup lang="ts">
// The personal feed: everything new in my groups since I last looked. Reading
// it advances the mark, so it is deliberately a single bounded batch — not an
// infinite timeline. Revisiting shows only what arrived since.
definePageMeta({ middleware: 'auth' })

const authFetch = useRequestFetch()
const { openComposer, posted } = useComposer()
const { data, refresh } = await useAsyncData('personal-feed', () =>
  authFetch<{ since: string; posts: any[] }>('/api/backend/feed')
)
// A new post from the global composer should show up here on return.
watch(posted, () => refresh())

const sinceLabel = computed(() =>
  data.value ? new Date(data.value.since).toLocaleString('fr-FR', {
    day: 'numeric', month: 'long', hour: '2-digit', minute: '2-digit'
  }) : ''
)

useHead({ title: 'Mon fil — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil"><Fa icon="house" /> Mon fil</p>
    <h1>Depuis votre dernière visite</h1>
    <p class="lede">Ce qui est arrivé dans vos groupes depuis le {{ sinceLabel }}.</p>

    <TransitionGroup v-if="data && data.posts.length" name="list" tag="div" class="feed">
      <PostItem v-for="p in data.posts" :key="p.id" :post="p" />
    </TransitionGroup>

    <div v-else class="empty">
      <Fa icon="seedling" class="fa-icon" />
      <p>Rien de neuf pour l'instant. Ce fil ne montre que ce qui est arrivé
        depuis votre dernier passage.</p>
      <div class="actions">
        <button class="btn accent" @click="openComposer()"><Fa icon="feather" /> Publier une observation</button>
        <NuxtLink class="btn ghost" to="/groupes"><Fa icon="users" /> Rejoindre un groupe</NuxtLink>
      </div>
    </div>
  </div>
</template>
