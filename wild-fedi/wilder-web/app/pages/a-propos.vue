<script setup lang="ts">
// Page d'explication, dans le ton du README wild-fedi : ce client ne « possède »
// rien, il lit un réseau. On y rappelle les idées d'ActivityPub et la façon dont
// ce client les traduit à l'écran.
const { wilderUrl } = useRuntimeConfig() // côté SSR : sert seulement à l'affichage
const { homeHandle } = useRuntimeConfig().public

const contrats = [
  { t: 'Acteur', d: "Le document d'un participant (école, classe, station) : ses inbox/outbox, sa clé publique." },
  { t: 'WebFinger', d: 'Transforme un handle @nom@domaine en URL d\'acteur — pourquoi les identités ressemblent à des e-mails.' },
  { t: 'Outbox', d: 'La liste de ce qu\'un acteur a publié : ici, le fil des observations.' },
  { t: 'Abonnés', d: 'Qui suit un acteur — la collection qui alimente les timelines.' },
  { t: 'Inbox (signée)', d: 'Où arrivent les messages des autres serveurs, chacun signé et vérifié.' },
  { t: 'NodeInfo', d: 'La carte d\'identité de l\'instance : logiciel, protocoles, nombre d\'acteurs.' }
]
</script>

<template>
  <div class="wrap">
    <p class="oeil">À propos</p>
    <h1>Un client, pas une plateforme</h1>
    <p class="lede">
      Wilder-web ne stocke aucune donnée et ne possède aucun contenu. Il lit, via le
      protocole <strong>ActivityPub</strong>, ce qu'une instance du réseau LICHEN publie —
      et l'affiche de deux façons à la fois : lisible, et brute.
    </p>

    <h2>Ce que vous voyez</h2>
    <p>
      Le fil montre les observations comme des cartes — texte, photos, espèce (hashtags),
      lieu géolocalisé, date. Sous chacune, « JSON ActivityPub » révèle l'objet <code>Create</code>
      tel qu'il circule sur le réseau. C'est le <strong>double niveau de lecture</strong> :
      une appli grand public y verrait un billet géolocalisé ; LICHEN y lit une observation
      structurée, cartographiable et rattachable à un sujet d'étude.
    </p>

    <h2>Les contrats qu'il consomme</h2>
    <div class="cartes">
      <article v-for="c in contrats" :key="c.t" class="carte">
        <h3>{{ c.t }}</h3>
        <p>{{ c.d }}</p>
      </article>
    </div>

    <h2>Comment il est branché</h2>
    <div class="note">
      <p>
        Le navigateur ne parle <strong>jamais</strong> directement au serveur ActivityPub.
        Chaque requête passe par les routes Nitro de ce site (<code>/api/*</code>), qui
        interrogent l'instance côté serveur. Cela évite les blocages CORS et permet
        d'envoyer le bon en-tête <code>Accept: application/activity+json</code>, sans lequel
        beaucoup de serveurs refusent de répondre en ActivityPub.
      </p>
    </div>

    <h2>Rester « sauvage »</h2>
    <p>
      Aucun traceur, aucun cookie, aucune police ni image distante — le fond animé est
      dessiné dans le navigateur, pas téléchargé. La carte est schématique, sans tuiles
      d'un tiers. L'idée est la même que pour le réseau lui-même : ne dépendre de personne.
    </p>

    <dl class="dl">
      <div><dt>Instance</dt><dd class="mono">{{ wilderUrl }}</dd></div>
      <div><dt>Acteur d'accueil</dt><dd class="mono">{{ homeHandle }}</dd></div>
    </dl>

    <p style="margin-top:1.4rem">
      <NuxtLink class="btn" to="/">Voir le fil</NuxtLink>
      <NuxtLink class="btn ghost" :to="{ path: '/acteur', query: { h: homeHandle } }" style="margin-left:.6rem">Ouvrir l'acteur de démo</NuxtLink>
    </p>
  </div>
</template>

<style scoped>
.cartes {
  display: grid; gap: 1rem; margin: .6rem 0 0;
  grid-template-columns: repeat(auto-fit, minmax(15rem, 1fr));
}
.carte {
  background: var(--paper); border: 1px solid var(--line);
  border-left: 2px solid var(--lichen); border-radius: var(--radius);
  padding: 1rem 1.15rem;
}
.carte h3 { margin: 0 0 .35rem; font-size: 1rem; }
.carte p { margin: 0; font-size: .9rem; color: var(--ink-soft); }
</style>
