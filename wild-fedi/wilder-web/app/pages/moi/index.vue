<script setup lang="ts">
// Personal space — profile management (avatar, display name, bio) plus
// shortcuts. Notes, research and the determination key arrive in a later phase.
definePageMeta({ middleware: 'auth' })

const { user } = useAuth()
const { homeHandle } = useRuntimeConfig().public
// The instance domain, taken from the configured home handle (name@domain).
const domain = String(homeHandle).split('@')[1]

const roleLabel: Record<string, string> = {
  school: 'École · administration',
  student: 'Élève',
  service: 'Station de capture'
}

// Load the actor document for the current avatar and bio (served outside the
// /api/v1 surface, via the existing actor proxy).
const authFetch = useRequestFetch()
const { data: actor, refresh: refreshActor } = await useAsyncData('me-actor', () =>
  user.value
    ? authFetch<any>('/api/actor', { query: { h: `${user.value.username}@${domain}` } })
    : Promise.resolve(null)
)

const avatarUrl = computed(() => actor.value?.icon?.url || null)

const form = reactive({ display_name: '', summary: '' })
watchEffect(() => {
  form.display_name = actor.value?.name || user.value?.display_name || ''
  form.summary = actor.value?.summary || ''
})

const savingProfile = ref(false)
const uploadingAvatar = ref(false)
const notice = ref('')
let noticeTimer: ReturnType<typeof setTimeout> | undefined

// A short-lived confirmation chip rather than a permanent line of text.
function flash(msg: string) {
  notice.value = msg
  clearTimeout(noticeTimer)
  noticeTimer = setTimeout(() => (notice.value = ''), 3200)
}

async function saveProfile() {
  savingProfile.value = true
  try {
    await $fetch('/api/backend/me/profile', {
      method: 'PATCH',
      body: { display_name: form.display_name || undefined, summary: form.summary || undefined }
    })
    await refreshActor()
    flash('Profil enregistré.')
  } finally {
    savingProfile.value = false
  }
}

async function onPickAvatar(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploadingAvatar.value = true
  try {
    await $fetch('/api/backend/me/avatar', {
      method: 'POST',
      body: file,
      headers: { 'content-type': file.type }
    })
    await refreshActor()
  } finally {
    uploadingAvatar.value = false
  }
}

useHead({ title: 'Mon espace — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil"><Fa icon="user" /> Mon espace</p>
    <h1>{{ user?.display_name || user?.username }}</h1>
    <p class="lede">Votre profil et votre espace personnel sur le réseau.</p>

    <section class="profile rise">
      <div class="avatar">
        <img v-if="avatarUrl" :src="mediaSrc(avatarUrl)" alt="avatar" />
        <div v-else class="avatar-ph" aria-hidden="true">{{ (user?.display_name || user?.username || '?').charAt(0).toUpperCase() }}</div>
        <label class="btn ghost sm" :class="{ loading: uploadingAvatar }">
          Changer l’avatar
          <input type="file" accept="image/*" hidden @change="onPickAvatar" />
        </label>
      </div>

      <form class="fields" @submit.prevent="saveProfile">
        <label>Nom affiché
          <input v-model="form.display_name" type="text" />
        </label>
        <label>Bio
          <textarea v-model="form.summary" rows="3" placeholder="Quelques mots sur vous…" />
        </label>
        <div class="actions">
          <button class="btn" type="submit" :class="{ loading: savingProfile }" :disabled="savingProfile">Enregistrer</button>
          <Transition name="list">
            <span v-if="notice" class="toast">{{ notice }}</span>
          </Transition>
        </div>
      </form>
    </section>

    <dl class="dl">
      <div><dt>Identifiant</dt><dd class="mono">@{{ user?.username }}</dd></div>
      <div><dt>Rôle</dt><dd>{{ roleLabel[user?.role || ''] || user?.role }}</dd></div>
    </dl>

    <h2>Raccourcis</h2>
    <div class="soon stagger">
      <NuxtLink class="card" to="/moi/feed"><h3><Fa icon="house" /> Mon fil</h3><p>Ce qui a changé dans vos groupes depuis votre dernière visite.</p></NuxtLink>
      <NuxtLink class="card" to="/moi/notes"><h3><Fa icon="book-open" /> Mes notes</h3><p>Vos notes privées, visibles de vous seul.</p></NuxtLink>
      <NuxtLink class="card" to="/moi/recherche"><h3><Fa icon="flask" /> Ma recherche</h3><p>Vos documents de recherche personnels.</p></NuxtLink>
      <NuxtLink class="card" to="/determiner"><h3><Fa icon="key" /> Clé de détermination</h3><p>Identifier une espèce du général au précis.</p></NuxtLink>
      <NuxtLink class="card" to="/groupes"><h3><Fa icon="users" /> Mes groupes</h3><p>Rejoindre un cercle d'étude ou en créer un.</p></NuxtLink>
    </div>
  </div>
</template>

<style scoped>
.profile { display: flex; gap: 1.5rem; flex-wrap: wrap; margin: 1.2rem 0 1rem; }
.avatar { display: flex; flex-direction: column; align-items: center; gap: .5rem; }
.avatar img, .avatar-ph {
  width: 6rem; height: 6rem; border-radius: 50%; object-fit: cover;
  border: 1px solid var(--line); background: var(--lichen-wash);
}
.avatar-ph { display: grid; place-items: center; font-size: 2rem; color: var(--lichen); }
.fields { flex: 1 1 18rem; display: flex; flex-direction: column; gap: .7rem; }
.fields label { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: var(--ink-soft); font-weight: 500; }
.soon { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr)); }
.soon h3 { margin: 0 0 .3rem; font-size: 1.02rem; display: flex; align-items: center; gap: .5rem; }
.soon h3 svg { color: var(--primary); }
.soon p { margin: 0; font-size: .9rem; color: var(--ink-soft); }
</style>
