<script setup lang="ts">
const annee = new Date().getFullYear()
const { instanceName, homeHandle } = useRuntimeConfig().public
</script>

<template>
  <div>
    <header class="top">
      <div class="wrap-wide top-in">
        <NuxtLink to="/" class="brand">Wilder <span>· observations naturalistes fédérées</span></NuxtLink>
        <nav>
          <NuxtLink to="/">Le fil</NuxtLink>
          <NuxtLink to="/carte">Carte</NuxtLink>
          <NuxtLink :to="{ path: '/acteur', query: { h: homeHandle } }">Acteur</NuxtLink>
          <NuxtLink to="/instance">Instance</NuxtLink>
          <NuxtLink to="/a-propos">À propos</NuxtLink>
        </nav>
      </div>
    </header>

    <main>
      <slot />
    </main>

    <footer class="foot">
      <div class="wrap-wide">
        <nav>
          <NuxtLink to="/">Le fil des observations</NuxtLink>
          <NuxtLink to="/carte">Carte des relevés</NuxtLink>
          <NuxtLink to="/instance">Instance {{ instanceName }}</NuxtLink>
          <NuxtLink to="/a-propos">À propos &amp; protocole</NuxtLink>
        </nav>
        <p>
          Client fédéré de démonstration pour le réseau LICHEN · Parle ActivityPub ·
          Aucun traceur, aucun cookie · Le navigateur ne contacte jamais directement le serveur.
        </p>
        <p>© {{ annee }} Projet LICHEN · Contenu sous licence CC BY 4.0.</p>
      </div>
    </footer>
  </div>
</template>
