<script setup lang="ts">
// Recherche d'un acteur par handle (@nom@domaine) ou par URL, puis navigation
// vers sa page. C'est la découverte fédérée : le même formulaire résout un
// acteur local ou n'importe quel acteur d'une autre instance.
const props = withDefaults(defineProps<{ value?: string }>(), { value: '' })
const query = ref(props.value)
const router = useRouter()

function submit() {
  const v = query.value.trim()
  if (!v) return
  // Une valeur qui ressemble à une URL est passée telle quelle ; sinon on la
  // traite comme un handle à résoudre par WebFinger côté serveur.
  const key = /^https?:\/\//.test(v) ? 'url' : 'h'
  router.push({ path: '/acteur', query: { [key]: v } })
}
</script>

<template>
  <form class="lookup" @submit.prevent="submit">
    <input
      v-model="query"
      type="search"
      placeholder="@beaver-class@127.0.0.1:8080  ·  ou une URL d'acteur"
      aria-label="Handle ou URL d'un acteur fédéré"
      spellcheck="false"
      autocapitalize="off"
    />
    <button class="btn" type="submit">Ouvrir</button>
  </form>
</template>
