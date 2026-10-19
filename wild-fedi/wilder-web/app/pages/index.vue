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
  <div class="wrap">
    <!-- Bandeau d'accueil ---------------------------------------------------->
    <section class="hero rise">
      <FondLichen />
      <div class="hero-in">
        <p class="oeil"><Fa icon="globe" /> Activité du réseau LICHEN</p>
        <h1 class="titre">Le carnet partagé du vivant</h1>
        <p class="accroche">
          Les observations naturalistes que les écoles du réseau publient et
          partagent, via ActivityPub — photos, espèces, lieux, relevés.
        </p>
        <div class="hero-lookup">
          <LookupForm :value="homeHandle" />
        </div>
      </div>
    </section>

    <!-- Le fil --------------------------------------------------------------->
    <section>
      <div v-if="pending" class="feed" aria-busy="true" aria-label="Chargement du fil">
        <div v-for="n in 3" :key="n" class="observation skelcard">
          <div class="skel skel-line" style="width:35%"></div>
          <div class="skel skel-line" style="width:92%"></div>
          <div class="skel skel-line" style="width:78%"></div>
          <div class="skel skel-card" style="margin-top:.7rem"></div>
        </div>
      </div>

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

        <div v-if="items.length" class="feed stagger">
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
/* Bandeau arrondi, contenu dans la colonne de lecture. */
.hero {
  position: relative; overflow: hidden;
  border: 1px solid var(--line); border-radius: var(--r-xl);
  padding: 2.6rem 2rem 2.4rem;
  background:
    radial-gradient(120% 120% at 100% 0%, var(--sage-wash) 0%, transparent 60%),
    var(--surface);
  box-shadow: var(--shadow);
}
.hero-in { position: relative; z-index: 1; max-width: 34rem; }
.titre {
  font-size: clamp(1.9rem, 1.2rem + 3vw, 2.9rem);
  line-height: 1.08; color: var(--forest-ink); margin: .2rem 0 .8rem;
}
.accroche { font-size: 1.05rem; color: var(--ink-soft); line-height: 1.5; max-width: 32rem; margin: 0 0 1.4rem; }
.hero-lookup { max-width: 30rem; }

.feed-head { margin-top: 2.4rem; }
.feed-head h2 { margin: 0 0 .3rem; }
.skelcard { pointer-events: none; }
.skelcard:hover { box-shadow: var(--shadow); border-color: var(--line); }
</style>
