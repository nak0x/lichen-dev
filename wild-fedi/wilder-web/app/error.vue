<script setup lang="ts">
defineProps<{ error: { statusCode: number; statusMessage?: string } }>()
useHead({ meta: [{ name: 'robots', content: 'noindex' }] })
</script>

<template>
  <div class="errpage">
    <div class="errcard card">
      <Fa :icon="error.statusCode === 404 ? 'binoculars' : 'circle-info'" class="big" />
      <p class="oeil">Erreur {{ error.statusCode }}</p>
      <h1>{{ error.statusCode === 404 ? 'Page introuvable' : 'Une erreur est survenue' }}</h1>
      <p class="lede">
        {{ error.statusCode === 404
          ? "Cette page n'existe pas — ou plus. Revenez sur le fil pour reprendre votre chemin."
          : (error.statusMessage || "Réessayez dans un instant.") }}
      </p>
      <NuxtLink class="btn" to="/"><Fa icon="house" /> Revenir à l'activité</NuxtLink>
    </div>
  </div>
</template>

<style scoped>
.errpage { min-height: 100svh; display: grid; place-items: center; padding: 2rem 1.25rem;
  background: radial-gradient(130% 80% at 50% -10%, var(--sage-wash) 0%, transparent 60%), var(--bg); }
.errcard { max-width: 28rem; text-align: center; border-radius: var(--r-xl); }
.errcard .big { font-size: 2.4rem; color: var(--sage); margin-bottom: .6rem; }
.errcard .oeil { justify-content: center; }
.errcard .btn { margin-top: 1rem; }
</style>
