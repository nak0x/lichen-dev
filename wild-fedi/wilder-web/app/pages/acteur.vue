<script setup lang="ts">
// Vue détaillée d'un acteur — local ou distant. Prend un handle (?h=) ou une URL
// (?url=), résout, puis charge en parallèle l'outbox et les abonnés. Cliquer un
// abonné rouvre la même page sur son acteur : c'est la fédération qui se parcourt.
const route = useRoute()

const { data, pending, error } = await useAsyncData(
  'actor-view',
  async () => {
    const actor = await $fetch<AnyObject>('/api/actor', {
      query: { h: route.query.h, url: route.query.url }
    })
    const empty = { totalItems: 0, items: [] as AnyObject[] }
    const [outbox, followers] = await Promise.all([
      actor.outbox ? $fetch<typeof empty>('/api/collection', { query: { url: actor.outbox } }) : empty,
      actor.followers ? $fetch<typeof empty>('/api/collection', { query: { url: actor.followers } }) : empty
    ])
    return { actor, outbox, followers }
  },
  { watch: [() => route.query.h, () => route.query.url] }
)

const actor = computed(() => data.value?.actor)
const observations = computed(() => data.value?.outbox.items ?? [])
const followers = computed(() => data.value?.followers.items ?? [])

useHead({ title: () => (actor.value ? `${handleOf(actor.value)} — Wilder` : 'Acteur — Wilder') })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Acteur fédéré</p>
    <LookupForm :value="typeof route.query.h === 'string' ? route.query.h : ''" />

    <div v-if="pending" class="spin" style="margin-top:1.4rem">Résolution de l'acteur…</div>

    <div v-else-if="error" class="err" style="margin-top:1.4rem">
      <p><strong>Acteur introuvable.</strong></p>
      <p>Le handle ou l'URL n'a pas pu être résolu. Vérifiez l'instance et l'orthographe.</p>
    </div>

    <template v-else-if="actor">
      <div style="margin-top:1.4rem">
        <ActorProfile
          :actor="actor"
          :outbox-total="data?.outbox.totalItems"
          :followers-total="data?.followers.totalItems"
        />
      </div>

      <h2>Observations</h2>
      <div v-if="observations.length" class="feed">
        <ObservationCard v-for="(it, i) in observations" :key="it.id || i" :item="it" />
      </div>
      <div v-else class="empty">Aucune publication dans l'outbox.</div>

      <h2>Abonnés</h2>
      <div v-if="followers.length" class="tags">
        <NuxtLink
          v-for="f in followers"
          :key="iriOf(f)"
          class="tag"
          :to="{ path: '/acteur', query: { url: iriOf(f) } }"
        >{{ iriOf(f) }}</NuxtLink>
      </div>
      <div v-else class="empty">Personne ne suit encore cet acteur.</div>
    </template>
  </div>
</template>
