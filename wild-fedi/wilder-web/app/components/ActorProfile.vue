<script setup lang="ts">
// Fiche d'un acteur : identité fédérée (handle, type), résumé, et les points de
// contact du protocole (inbox/outbox/followers, clé publique). Le JSON brut
// reste accessible pour voir à quoi ressemble un « acteur » sur le réseau.
const props = defineProps<{ actor: AnyObject; followersTotal?: number; outboxTotal?: number }>()

const handle = computed(() => handleOf(props.actor))
const typeLabel = computed(() => actorTypeLabel(props.actor?.type))
const summary = computed(() => toPlainText(props.actor?.summary))
const keyId = computed(() => props.actor?.publicKey?.id)
const sharedInbox = computed(() => props.actor?.endpoints?.sharedInbox)
</script>

<template>
  <section class="actor">
    <div class="actor-top">
      <h1>{{ actor.name || actor.preferredUsername || 'Acteur' }}</h1>
      <span class="badge">{{ typeLabel }}</span>
    </div>
    <p class="handle">{{ handle }}</p>

    <p v-if="summary" class="summary">{{ summary }}</p>

    <div class="actor-stats">
      <div>
        <b>{{ outboxTotal ?? '—' }}</b>
        <span>observations</span>
      </div>
      <div>
        <b>{{ followersTotal ?? '—' }}</b>
        <span>abonnés</span>
      </div>
    </div>

    <dl class="dl">
      <div><dt>Identité</dt><dd class="mono">{{ actor.id }}</dd></div>
      <div v-if="actor.inbox"><dt>Inbox</dt><dd class="mono">{{ actor.inbox }}</dd></div>
      <div v-if="actor.outbox"><dt>Outbox</dt><dd class="mono">{{ actor.outbox }}</dd></div>
      <div v-if="actor.followers"><dt>Abonnés</dt><dd class="mono">{{ actor.followers }}</dd></div>
      <div v-if="sharedInbox"><dt>Inbox partagée</dt><dd class="mono">{{ sharedInbox }}</dd></div>
      <div v-if="keyId"><dt>Clé publique</dt><dd class="mono">{{ keyId }}</dd></div>
    </dl>

    <RawJson :data="actor" label="JSON de l'acteur" />
  </section>
</template>
