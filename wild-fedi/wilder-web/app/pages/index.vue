<script setup lang="ts">
// Le fil : les observations publiées par l'acteur d'accueil (l'outbox de wilder).
// On enchaîne deux appels serveur — l'acteur, puis son outbox — pour reconstruire
// exactement le chemin qu'un serveur fédéré suit avant d'afficher un profil.
const { homeHandle, instanceName } = useRuntimeConfig().public

const { data, pending, error } = await useAsyncData('home-feed', async () => {
  const actor = await $fetch<AnyObject>('/api/actor')
  const outbox = await $fetch<{ totalItems: number; items: AnyObject[] }>('/api/collection', {
    query: { url: actor.outbox }
  })
  return { actor, outbox }
})

const actor = computed(() => data.value?.actor)
const items = computed(() => data.value?.outbox.items ?? [])

useHead({ title: `Wilder — le fil de ${instanceName}` })
</script>

<template>
  <div>
    <!-- Hero ----------------------------------------------------------------->
    <section class="hero">
      <FondLichen />
      <div class="hero-in">
        <p class="oeil">Réseau LICHEN · fédiverse</p>
        <h1 class="titre">Wilder</h1>
        <p class="accroche">
          Un client fédéré qui lit, via ActivityPub, les observations naturalistes
          qu'une école partage avec le réseau — photos, notes, espèces, lieux.
        </p>
        <div class="hero-lookup">
          <LookupForm :value="homeHandle" />
          <p class="fine">
            Cherchez un acteur par son handle <code>@nom@domaine</code> — local ou distant.
          </p>
        </div>
      </div>
    </section>

    <!-- Le fil --------------------------------------------------------------->
    <section class="wrap">
      <div v-if="pending" class="spin">Chargement du fil…</div>

      <div v-else-if="error" class="err">
        <p><strong>Impossible de joindre l'instance.</strong></p>
        <p>
          Vérifiez que le serveur <code>wilder</code> tourne et est atteignable
          (par défaut <code>http://127.0.0.1:8080</code>, réglable via <code>WILDER_URL</code>).
        </p>
      </div>

      <template v-else>
        <header v-if="actor" class="feed-head">
          <p class="oeil">Le fil</p>
          <h2>{{ actor.name || actor.preferredUsername }}</h2>
          <p class="lede">
            {{ items.length }} observation{{ items.length > 1 ? 's' : '' }} publiée{{ items.length > 1 ? 's' : '' }} ·
            <NuxtLink :to="{ path: '/acteur', query: { url: actor.id } }">voir le profil complet</NuxtLink>
          </p>
        </header>

        <div v-if="items.length" class="feed">
          <ObservationCard v-for="(it, i) in items" :key="it.id || i" :item="it" />
        </div>
        <div v-else class="empty">
          Cet acteur n'a encore rien publié.
        </div>

        <div class="note" style="margin-top:1.6rem">
          <p>
            Chaque observation ci-dessus est un objet <code>Create</code> transportant une
            <code>Note</code> — le même JSON qu'un autre serveur du fédiverse recevrait.
            Dépliez « JSON ActivityPub » sous une carte pour lire le fil brut.
          </p>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
/* La page déborde le rembourrage de <main> pour que le hero touche l'en-tête. */
.hero {
  position: relative;
  overflow: hidden;
  margin-top: -2.5rem;
  padding: 4rem 0 3.2rem;
  border-bottom: 1px solid var(--line);
  background:
    radial-gradient(120% 90% at 50% 0%, var(--lichen-wash) 0%, transparent 70%),
    var(--bg);
}
.hero-in { position: relative; z-index: 1; max-width: 44rem; margin: 0 auto; padding: 0 1.25rem; text-align: center; }
.titre {
  font-size: clamp(2.8rem, 11vw, 5.5rem);
  line-height: 1.05; letter-spacing: .03em; color: var(--forest-ink);
  margin: .3rem 0 1.1rem;
}
.accroche {
  font-family: var(--serif); font-size: clamp(1.05rem, 2.2vw, 1.3rem);
  line-height: 1.45; max-width: 32rem; margin: 0 auto 1.8rem;
}
.hero-lookup { max-width: 30rem; margin: 0 auto; text-align: left; }
.hero-lookup .fine { font-size: .8rem; color: var(--ink-soft); margin: .5rem 0 0; text-align: center; }

.feed-head { margin-top: 2.4rem; }
.feed-head h2 { margin: 0 0 .3rem; }
</style>
