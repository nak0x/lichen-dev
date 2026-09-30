<script setup lang="ts">
// « Lire le fil » (README §7, étape 3) : chaque objet peut révéler son JSON
// ActivityPub brut. C'est la meilleure façon de comprendre que l'affichage
// lisible et la structure sur le réseau sont une seule et même donnée.
const props = defineProps<{ data: unknown; label?: string }>()
const pretty = computed(() => JSON.stringify(props.data, null, 2))
</script>

<template>
  <details class="raw">
    <summary>{{ label || 'JSON ActivityPub' }}</summary>
    <pre>{{ pretty }}</pre>
  </details>
</template>
