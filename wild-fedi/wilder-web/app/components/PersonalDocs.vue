<script setup lang="ts">
// Reusable CRUD for a kind of private personal document (note or research).
// Items are reactive, so each row's textarea edits in place; Save sends a PATCH.
const props = defineProps<{ kind: 'note' | 'research'; withTitle?: boolean }>()

const authFetch = useRequestFetch()
const { data: docs, refresh } = await useAsyncData(`docs-${props.kind}`, () =>
  authFetch<any[]>(`/api/backend/notes?kind=${props.kind}`)
)

const draft = reactive({ title: '', content: '' })
const busy = ref(false)

async function create() {
  if (!draft.content.trim()) return
  busy.value = true
  try {
    await $fetch('/api/backend/notes', {
      method: 'POST',
      body: { kind: props.kind, title: draft.title || undefined, content: draft.content }
    })
    draft.title = ''
    draft.content = ''
    await refresh()
  } finally {
    busy.value = false
  }
}

async function save(doc: any) {
  await $fetch(`/api/backend/notes/${doc.id}`, {
    method: 'PATCH',
    body: { title: doc.title || undefined, content: doc.content }
  })
  await refresh()
}

async function remove(id: string) {
  await $fetch(`/api/backend/notes/${id}`, { method: 'DELETE' })
  await refresh()
}
</script>

<template>
  <div>
    <form class="new card" @submit.prevent="create">
      <input v-if="withTitle" v-model="draft.title" type="text" placeholder="Titre" />
      <textarea v-model="draft.content" rows="3" :placeholder="withTitle ? 'Votre recherche…' : 'Une note…'" required />
      <div class="actions">
        <button class="btn" type="submit" :disabled="busy">{{ busy ? '…' : 'Ajouter' }}</button>
      </div>
    </form>

    <div v-if="docs && docs.length" class="list">
      <article v-for="doc in docs" :key="doc.id" class="doc card">
        <input v-if="withTitle" v-model="doc.title" type="text" placeholder="Titre" />
        <textarea v-model="doc.content" rows="3" />
        <div class="actions">
          <button class="btn sm" @click="save(doc)">Enregistrer</button>
          <button class="btn danger sm" @click="remove(doc.id)">Supprimer</button>
          <span class="meta">modifié {{ new Date(doc.updated_at).toLocaleString('fr-FR') }}</span>
        </div>
      </article>
    </div>
    <div v-else class="empty">Rien encore. Ajoutez-en un ci-dessus.</div>
  </div>
</template>

<style scoped>
.new { margin-bottom: 1.2rem; display: flex; flex-direction: column; gap: .5rem; }
.new input, .doc input { width: 100%; }
.new textarea, .doc textarea { width: 100%; }
.list { display: flex; flex-direction: column; gap: 1rem; }
.doc { display: flex; flex-direction: column; gap: .5rem; }
.doc .actions { gap: .5rem; }
</style>
