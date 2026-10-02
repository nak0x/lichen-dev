<script setup lang="ts">
// Public actualités: the school's events and news, read-only for everyone.
definePageMeta({ layout: 'app', middleware: 'auth' })

const authFetch = useRequestFetch()
const { data: events } = await useAsyncData('events', () => authFetch<any[]>('/api/backend/events'))
const { data: news } = await useAsyncData('news', () => authFetch<any[]>('/api/backend/news'))

function when(iso: string | null) {
  return iso
    ? new Date(iso).toLocaleString('fr-FR', { weekday: 'long', day: 'numeric', month: 'long', hour: '2-digit', minute: '2-digit' })
    : ''
}
function day(iso: string) {
  return new Date(iso).toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })
}

useHead({ title: 'Actualités — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Le réseau</p>
    <h1>Actualités &amp; événements</h1>

    <h2>Événements à venir</h2>
    <div v-if="events && events.length" class="list">
      <article v-for="e in events" :key="e.id" class="card event">
        <h3>{{ e.title }}</h3>
        <p v-if="e.start_time" class="meta">◈ {{ when(e.start_time) }}<template v-if="e.location"> · {{ e.location }}</template></p>
        <p class="body">{{ e.content }}</p>
        <RawJson :data="e.activity" />
      </article>
    </div>
    <div v-else class="empty">Aucun événement annoncé.</div>

    <h2>Actualités</h2>
    <div v-if="news && news.length" class="list">
      <article v-for="n in news" :key="n.id" class="card">
        <h3>{{ n.title }}</h3>
        <p class="meta">{{ day(n.published) }}</p>
        <p class="body">{{ n.content }}</p>
        <RawJson :data="n.activity" />
      </article>
    </div>
    <div v-else class="empty">Aucune actualité.</div>
  </div>
</template>

<style scoped>
.list { display: flex; flex-direction: column; gap: 1rem; }
.card h3 { margin: 0 0 .2rem; }
.card .body { white-space: pre-wrap; margin: .5rem 0; }
.event { border-left: 2px solid var(--forest); }
</style>
