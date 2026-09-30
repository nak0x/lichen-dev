<script setup lang="ts">
// Rend une entrée d'outbox comme une observation lisible — tout en gardant le
// JSON brut à un clic. C'est le « double niveau de lecture » du README §5 : une
// appli grand public y voit un joli billet géolocalisé, LICHEN y lit une
// observation structurée (espèce, coordonnées, date, média).
const props = defineProps<{ item: AnyObject }>()

const verb = computed(() => props.item?.type || 'Objet')
const obj = computed(() => objectOf(props.item))
const published = computed(() => props.item?.published || obj.value?.published)
const content = computed(() => toPlainText(obj.value?.content))
const images = computed(() => imagesOf(obj.value))
const place = computed(() => placeOf(obj.value))
const tags = computed(() => hashtagsOf(obj.value))
const objId = computed(() => obj.value?.id || props.item?.id || '')
const isNote = computed(() => obj.value?.type === 'Note' || obj.value?.type === 'Article')

// Verbe en français, pour l'entête.
const verbLabel = computed(() => {
  switch (verb.value) {
    case 'Create': return 'Publication'
    case 'Announce': return 'Partage'
    case 'Update': return 'Mise à jour'
    default: return verb.value
  }
})
</script>

<template>
  <article class="observation">
    <div class="obs-head">
      <span class="verb">{{ verbLabel }}</span>
      <span class="dot">·</span>
      <span v-if="isNote">observation</span>
      <span v-else>{{ obj.type }}</span>
      <template v-if="published">
        <span class="dot">·</span>
        <time :datetime="published">{{ frDate(published) }}</time>
      </template>
      <span class="dot">·</span>
      <span>{{ isPublic(item) ? 'Public' : 'Restreint' }}</span>
    </div>

    <p v-if="content" class="obs-content" style="white-space: pre-wrap">{{ content }}</p>
    <p v-else class="obs-content meta"><em>Sans texte</em></p>

    <div v-if="images.length" class="obs-media">
      <figure v-for="(img, i) in images" :key="i">
        <img :src="img.url" :alt="img.name || 'pièce jointe'" loading="lazy" />
        <figcaption v-if="img.name">{{ img.name }}</figcaption>
      </figure>
    </div>

    <div v-if="place" class="place">
      <span class="pin" aria-hidden="true">◈</span>
      <span>{{ place.name || 'Lieu du relevé' }}</span>
      <span class="coord">{{ place.lat.toFixed(4) }}, {{ place.lon.toFixed(4) }}</span>
    </div>

    <div v-if="tags.length" class="tags">
      <span v-for="t in tags" :key="t" class="tag hash">{{ t }}</span>
    </div>

    <div class="obs-foot">
      <span class="mono">{{ objId }}</span>
    </div>

    <RawJson :data="item" />
  </article>
</template>
