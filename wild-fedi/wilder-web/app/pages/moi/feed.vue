<script setup lang="ts">
// The personal feed: everything new in my groups since I last looked. Reading
// it advances the mark, so it is deliberately a single bounded batch — not an
// infinite timeline. Revisiting shows only what arrived since.
definePageMeta({ layout: 'app', middleware: 'auth' })

const authFetch = useRequestFetch()
const { data } = await useAsyncData('personal-feed', () =>
  authFetch<{ since: string; posts: any[] }>('/api/backend/feed')
)

const sinceLabel = computed(() =>
  data.value ? new Date(data.value.since).toLocaleString('fr-FR', {
    day: 'numeric', month: 'long', hour: '2-digit', minute: '2-digit'
  }) : ''
)

useHead({ title: 'Mon fil — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Mon fil</p>
    <h1>Depuis votre dernière visite</h1>
    <p class="lede">Nouveau dans vos groupes depuis le {{ sinceLabel }}.</p>

    <div v-if="data && data.posts.length" class="feed">
      <PostItem v-for="p in data.posts" :key="p.id" :post="p" />
    </div>
    <div v-else class="empty">
      Rien de neuf. Revenez après que vos groupes aient publié — ce fil ne montre
      que ce qui est arrivé depuis votre dernier passage.
    </div>
  </div>
</template>
