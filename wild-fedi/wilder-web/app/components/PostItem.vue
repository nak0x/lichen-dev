<script setup lang="ts">
// Renders one post from our own API (author, content, tags, images) with the raw
// ActivityPub `Create` a click away — the same dual reading as the public feed.
const props = defineProps<{ post: any }>()

function when(iso: string): string {
  return new Date(iso).toLocaleString('fr-FR', {
    day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit'
  })
}

// Image attachments carried on the post's embedded Note.
const images = computed<string[]>(() => {
  const att = props.post?.activity?.object?.attachment
  if (!Array.isArray(att)) return []
  return att.filter((a: any) => a?.type === 'Image' && a?.url).map((a: any) => a.url as string)
})
</script>

<template>
  <article class="observation">
    <div class="obs-head">
      <span class="verb">{{ post.author }}</span>
      <span class="dot">·</span>
      <time :datetime="post.published">{{ when(post.published) }}</time>
    </div>
    <p v-if="post.content" class="obs-content" style="white-space: pre-wrap">{{ post.content }}</p>
    <div v-if="images.length" class="obs-media">
      <img v-for="(src, i) in images" :key="i" :src="mediaSrc(src)" alt="pièce jointe" loading="lazy" />
    </div>
    <div v-if="post.tags?.length" class="tags">
      <span v-for="t in post.tags" :key="t" class="tag hash">#{{ t }}</span>
    </div>
    <RawJson :data="post.activity" />
  </article>
</template>

<style scoped>
.obs-media { display: grid; gap: .5rem; margin: .6rem 0; grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr)); }
.obs-media img { width: 100%; height: 10rem; object-fit: cover; border: 1px solid var(--line); border-radius: var(--radius); background: var(--bg); }
</style>
