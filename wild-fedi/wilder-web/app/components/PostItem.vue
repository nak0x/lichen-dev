<script setup lang="ts">
// Renders one post from our own API (author, content, tags) with the raw
// ActivityPub `Create` a click away — the same dual reading as the public feed.
defineProps<{ post: any }>()

function when(iso: string): string {
  return new Date(iso).toLocaleString('fr-FR', {
    day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit'
  })
}
</script>

<template>
  <article class="observation">
    <div class="obs-head">
      <span class="verb">{{ post.author }}</span>
      <span class="dot">·</span>
      <time :datetime="post.published">{{ when(post.published) }}</time>
    </div>
    <p v-if="post.content" class="obs-content" style="white-space: pre-wrap">{{ post.content }}</p>
    <div v-if="post.tags?.length" class="tags">
      <span v-for="t in post.tags" :key="t" class="tag hash">#{{ t }}</span>
    </div>
    <RawJson :data="post.activity" />
  </article>
</template>
