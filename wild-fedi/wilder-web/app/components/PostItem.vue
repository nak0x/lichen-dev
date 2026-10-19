<script setup lang="ts">
// One observation from our own API, rendered as a specimen card: who + when,
// text, photos, a place pin, and species tags as leafy chips — with the raw
// ActivityPub `Create` a click away (the same dual reading as the public feed).
const props = defineProps<{ post: any }>()

const initial = computed(() => (props.post?.author || '?').charAt(0).toUpperCase())

function when(iso: string): string {
  return new Date(iso).toLocaleString('fr-FR', {
    day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit'
  })
}

const images = computed<string[]>(() => {
  const att = props.post?.activity?.object?.attachment
  if (!Array.isArray(att)) return []
  return att.filter((a: any) => a?.type === 'Image' && a?.url).map((a: any) => a.url as string)
})

// A place, if the Note carried one.
const place = computed(() => {
  const loc = props.post?.activity?.object?.location
  if (!loc) return null
  return { name: loc.name as string | undefined, lat: loc.latitude, lon: loc.longitude }
})
</script>

<template>
  <article class="observation">
    <div class="obs-head">
      <span class="who"><span class="sb-avatar">{{ initial }}</span>{{ post.author }}</span>
      <span class="dot">·</span>
      <time :datetime="post.published">{{ when(post.published) }}</time>
    </div>

    <p v-if="post.content" class="obs-content" style="white-space: pre-wrap">{{ post.content }}</p>

    <div v-if="images.length" class="obs-media">
      <figure v-for="(src, i) in images" :key="i">
        <img :src="mediaSrc(src)" alt="pièce jointe" loading="lazy" />
      </figure>
    </div>

    <div v-if="place" class="place">
      <Fa icon="location-dot" />
      <span>{{ place.name || 'Lieu du relevé' }}</span>
      <span v-if="place.lat != null" class="coord">{{ Number(place.lat).toFixed(3) }}, {{ Number(place.lon).toFixed(3) }}</span>
    </div>

    <div v-if="post.tags?.length" class="tags">
      <span v-for="t in post.tags" :key="t" class="tag hash"><Fa icon="leaf" /> {{ t }}</span>
    </div>

    <RawJson :data="post.activity" />
  </article>
</template>
