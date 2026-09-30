<script setup lang="ts">
// Carte des relevés. Volontairement schématique : les points sont projetés dans
// un SVG dessiné à l'exécution, sans fond de carte ni tuiles distantes — fidèle
// au parti pris « aucune requête vers un tiers » du projet. On lit les
// coordonnées d'un coup d'œil et le détail dans la liste.
const { instanceName } = useRuntimeConfig().public

const { data, pending, error } = await useAsyncData('carte', async () => {
  const actor = await $fetch<AnyObject>('/api/actor')
  const outbox = await $fetch<{ totalItems: number; items: AnyObject[] }>('/api/collection', {
    query: { url: actor.outbox }
  })
  return { actor, items: outbox.items }
})

// On ne garde que les observations qui portent un lieu géolocalisé.
const located = computed(() => {
  return (data.value?.items ?? [])
    .map((item) => ({ item, place: placeOf(objectOf(item)) }))
    .filter((x): x is { item: AnyObject; place: NonNullable<ReturnType<typeof placeOf>> } => !!x.place)
})

// Projection des lat/lon dans un repère 100×60, avec marge. Un seul point → centré.
const W = 100, H = 60, PAD = 12
const points = computed(() => {
  const pts = located.value
  if (!pts.length) return []
  const lons = pts.map((p) => p.place.lon)
  const lats = pts.map((p) => p.place.lat)
  const minLon = Math.min(...lons), maxLon = Math.max(...lons)
  const minLat = Math.min(...lats), maxLat = Math.max(...lats)
  const spanLon = maxLon - minLon || 0.02
  const spanLat = maxLat - minLat || 0.02
  return pts.map((p, i) => ({
    i,
    name: p.place.name,
    lat: p.place.lat,
    lon: p.place.lon,
    id: p.item.id,
    x: PAD + ((p.place.lon - minLon) / spanLon) * (W - 2 * PAD),
    // y inversé : la latitude croît vers le haut.
    y: PAD + (1 - (p.place.lat - minLat) / spanLat) * (H - 2 * PAD)
  }))
})

useHead({ title: 'Carte des relevés — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">Suivi naturaliste</p>
    <h1>Carte des relevés</h1>
    <p class="lede">
      Les observations géolocalisées de l'instance {{ instanceName }}, situées par leur
      champ <code>location</code> (un <code>Place</code> ActivityStreams).
    </p>

    <div v-if="pending" class="spin">Chargement…</div>
    <div v-else-if="error" class="err">Instance injoignable.</div>

    <template v-else>
      <div v-if="points.length" class="map">
        <svg :viewBox="`0 0 ${W} ${H}`" role="img" aria-label="Carte schématique des relevés">
          <rect class="field" x="1" y="1" :width="W - 2" :height="H - 2" rx="2" />
          <g v-for="p in points" :key="p.i">
            <circle class="pt" :cx="p.x" :cy="p.y" r="2.2" />
            <text class="pt-label" :x="p.x + 3.2" :y="p.y + 2.4">{{ p.i + 1 }}</text>
          </g>
        </svg>
      </div>
      <div v-else class="empty">Aucune observation géolocalisée pour l'instant.</div>

      <ol v-if="located.length" class="releves">
        <li v-for="(l, i) in located" :key="l.item.id || i">
          <span class="idx">{{ i + 1 }}</span>
          <span class="nom">{{ l.place.name || 'Lieu' }}</span>
          <span class="coord mono">{{ l.place.lat.toFixed(4) }}, {{ l.place.lon.toFixed(4) }}</span>
          <NuxtLink
            v-if="data?.actor"
            class="lien"
            :to="{ path: '/acteur', query: { url: data.actor.id } }"
          >observation</NuxtLink>
        </li>
      </ol>
    </template>
  </div>
</template>

<style scoped>
.releves { list-style: none; padding: 0; margin: 1.4rem 0 0; }
.releves li {
  display: flex; align-items: baseline; gap: .8rem; flex-wrap: wrap;
  padding: .6rem 0; border-top: 1px solid var(--line);
}
.idx {
  flex: none; width: 1.5rem; height: 1.5rem; border-radius: 50%;
  background: var(--forest); color: var(--paper);
  display: grid; place-items: center; font-size: .78rem;
}
@media (prefers-color-scheme: dark) { .idx { color: #12160f; } }
.nom { font-weight: 500; }
.coord { color: var(--ink-soft); }
.lien { margin-left: auto; font-size: .82rem; }
</style>
