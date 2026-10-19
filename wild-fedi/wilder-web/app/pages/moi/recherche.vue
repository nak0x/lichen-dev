<script setup lang="ts">
// Private personal research documents (title + body).
definePageMeta({ middleware: 'auth' })
useHead({ title: 'Ma recherche — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil"><Fa icon="flask" /> Espace personnel</p>
    <h1>Ma recherche</h1>
    <p class="lede">Vos documents de recherche, privés — un carnet à vous.</p>
    <PersonalDocs kind="research" :with-title="true" />
  </div>
</template>
