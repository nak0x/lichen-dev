<script setup lang="ts">
// Private self-notes.
definePageMeta({ middleware: 'auth' })
useHead({ title: 'Mes notes — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil"><Fa icon="book-open" /> Espace personnel</p>
    <h1>Mes notes</h1>
    <p class="lede">Des notes privées, visibles de vous seul — jamais fédérées.</p>
    <PersonalDocs kind="note" />
  </div>
</template>
