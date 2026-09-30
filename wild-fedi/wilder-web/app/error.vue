<script setup lang="ts">
defineProps<{ error: { statusCode: number; statusMessage?: string } }>()
useHead({ meta: [{ name: 'robots', content: 'noindex' }] })
</script>

<template>
  <div class="wrap" style="padding-top:4rem;padding-bottom:4rem">
    <p class="oeil">Erreur</p>
    <h1>{{ error.statusCode }}</h1>
    <p class="lede">
      {{ error.statusCode === 404 ? "Cette page n'existe pas." : (error.statusMessage || "Une erreur est survenue.") }}
    </p>
    <p><NuxtLink to="/">Revenir au fil</NuxtLink></p>
  </div>
</template>
