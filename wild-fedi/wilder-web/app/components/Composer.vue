<script setup lang="ts">
// The global "post an observation" modal. Opened from the sidebar or any empty
// state via useComposer(). Picks a group, writes text, optionally attaches a
// photo and tags, then posts. A guest's post is held for review — we say so
// rather than pretending it went live.
const { open, presetGroup, posted, closeComposer } = useComposer()

const groups = ref<any[]>([])
const draft = reactive({ group: '', content: '', tags: '' })
const attached = ref<{ id: string; url: string } | null>(null)
const uploading = ref(false)
const posting = ref(false)
const error = ref('')
const done = ref<'' | 'published' | 'pending'>('')

// Load the groups the user may post into, the first time the modal opens.
async function loadGroups() {
  try {
    groups.value = await $fetch<any[]>('/api/backend/groups/mine')
  } catch {
    groups.value = []
  }
}

watch(open, async (v) => {
  if (!v) return
  error.value = ''
  done.value = ''
  if (!groups.value.length) await loadGroups()
  draft.group = presetGroup.value || groups.value[0]?.username || ''
})

function reset() {
  draft.content = ''
  draft.tags = ''
  attached.value = null
}

async function onPick(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploading.value = true
  try {
    attached.value = await $fetch('/api/backend/media', {
      method: 'POST', body: file, headers: { 'content-type': file.type }
    })
  } finally {
    uploading.value = false
  }
}

async function submit() {
  if (!draft.group || !draft.content.trim()) return
  posting.value = true
  error.value = ''
  try {
    const post = await $fetch<{ status: string }>('/api/backend/posts', {
      method: 'POST',
      body: {
        group: draft.group,
        content: draft.content,
        tags: draft.tags.split(/[\s,]+/).filter(Boolean),
        media: attached.value ? [attached.value.id] : []
      }
    })
    posted.value++
    if (post.status === 'pending') {
      done.value = 'pending'
      reset()
    } else {
      const target = draft.group
      reset()
      closeComposer()
      await navigateTo(`/groupe/${target}`)
    }
  } catch (e: any) {
    error.value = e?.data?.error || 'Publication impossible. Réessayez.'
  } finally {
    posting.value = false
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') closeComposer()
}
</script>

<template>
  <Transition name="modal">
    <div v-if="open" class="composer-overlay" @keydown="onKey">
      <div class="composer-scrim" @click="closeComposer" />
      <div class="composer-panel card" role="dialog" aria-modal="true" aria-label="Publier une observation">
        <header class="composer-head">
          <h2><Fa icon="feather" /> Publier une observation</h2>
          <button class="iconbtn" aria-label="Fermer" @click="closeComposer"><Fa icon="xmark" /></button>
        </header>

        <div v-if="done === 'pending'" class="held">
          <p class="toast"><Fa icon="clock-rotate-left" /> Envoyée pour validation par un administrateur.</p>
          <p class="meta">En tant qu'invité, vos observations sont relues avant d'être publiées.</p>
          <div class="actions">
            <button class="btn ghost" @click="done = ''">Écrire une autre</button>
            <button class="btn" @click="closeComposer">Fermer</button>
          </div>
        </div>

        <form v-else-if="groups.length" @submit.prevent="submit">
          <label class="field">Dans le groupe
            <select v-model="draft.group" required>
              <option v-for="g in groups" :key="g.username" :value="g.username">{{ g.name }}</option>
            </select>
          </label>

          <label class="field">Votre observation
            <textarea v-model="draft.content" rows="4" placeholder="Qu'avez-vous observé ? Espèce, lieu, comportement…" required />
          </label>

          <Transition name="list">
            <figure v-if="attached" class="preview">
              <img :src="mediaSrc(attached.url)" alt="aperçu" />
              <button type="button" class="btn ghost sm" @click="attached = null"><Fa icon="trash" /> Retirer</button>
            </figure>
          </Transition>

          <div class="composer-foot">
            <input v-model="draft.tags" type="text" placeholder="étiquettes : castor nuit rivière" />
            <label class="btn ghost sm imgbtn" :class="{ loading: uploading }">
              <Fa icon="image" /> <span>{{ attached ? 'Photo ✓' : 'Photo' }}</span>
              <input type="file" accept="image/*" hidden @change="onPick" />
            </label>
            <button class="btn accent" type="submit" :class="{ loading: posting }" :disabled="posting || !draft.content.trim()">
              <Fa icon="feather" /> Publier
            </button>
          </div>
          <p v-if="error" class="err">{{ error }}</p>
        </form>

        <div v-else class="empty">
          <Fa icon="users" class="fa-icon" />
          <p>Rejoignez ou créez un groupe pour publier une observation.</p>
          <NuxtLink class="btn" to="/groupes" @click="closeComposer">Voir les groupes</NuxtLink>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.composer-overlay { position: fixed; inset: 0; z-index: 80; display: grid; place-items: start center; padding: 6vh 1rem 1rem; }
.composer-scrim { position: fixed; inset: 0; background: rgba(10, 14, 9, .5); }
.composer-panel { position: relative; width: 100%; max-width: 32rem; border-radius: var(--r-xl); box-shadow: var(--shadow-lift); }
.composer-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
.composer-head h2 { margin: 0; font-size: 1.25rem; display: inline-flex; align-items: center; gap: .55rem; }
.composer-head h2 svg { color: var(--accent); }
.iconbtn { font: inherit; background: none; border: 0; cursor: pointer; color: var(--ink-soft); font-size: 1.1rem; width: 2rem; height: 2rem; border-radius: var(--pill); display: grid; place-items: center; transition: background var(--fast) var(--ease), color var(--fast) var(--ease); }
.iconbtn:hover { background: var(--sage-wash); color: var(--ink); }
form { display: flex; flex-direction: column; gap: .9rem; }
.preview { margin: 0; display: flex; align-items: flex-start; gap: .7rem; }
.preview img { width: 8rem; height: 8rem; object-fit: cover; border-radius: var(--r); border: 1px solid var(--line); }
.composer-foot { display: flex; gap: .5rem; align-items: center; flex-wrap: wrap; }
.composer-foot input[type="text"] { flex: 1 1 10rem; }
.imgbtn { flex: none; cursor: pointer; }
.held { display: flex; flex-direction: column; gap: .7rem; align-items: flex-start; }

.modal-enter-active, .modal-leave-active { transition: opacity var(--med) var(--ease); }
.modal-enter-active .composer-panel { transition: transform var(--med) var(--ease); }
.modal-enter-from, .modal-leave-to { opacity: 0; }
.modal-enter-from .composer-panel { transform: translateY(14px) scale(.98); }
</style>
