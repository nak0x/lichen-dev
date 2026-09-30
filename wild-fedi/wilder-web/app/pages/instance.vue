<script setup lang="ts">
// Fiche de l'instance : ce que le serveur déclare dans NodeInfo 2.1 — le
// document qu'un autre serveur lit pour découvrir « qui parle » avant de fédérer.
const { data, pending, error } = await useAsyncData('nodeinfo', () =>
  $fetch<AnyObject>('/api/nodeinfo')
)

const software = computed(() => data.value?.software || {})
const usage = computed(() => data.value?.usage || {})
const metadata = computed(() => data.value?.metadata || {})
const protocols = computed(() => asArray<string>(data.value?.protocols))

useHead({ title: 'Instance — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil">NodeInfo</p>
    <h1>L'instance</h1>
    <p class="lede">
      Le point de découverte du fédiverse : chaque serveur publie ce qu'il fait tourner,
      pour qui, et selon quels protocoles.
    </p>

    <div v-if="pending" class="spin">Interrogation de l'instance…</div>
    <div v-else-if="error" class="err">
      <p><strong>NodeInfo indisponible.</strong></p>
      <p>Le serveur <code>wilder</code> est-il démarré et atteignable ?</p>
    </div>

    <template v-else-if="data">
      <div class="kpis" style="margin-bottom:1.4rem">
        <div class="kpi">
          <b>{{ software.name || '—' }}</b>
          <span>logiciel</span>
        </div>
        <div class="kpi">
          <b>{{ software.version || '—' }}</b>
          <span>version</span>
        </div>
        <div class="kpi">
          <b>{{ usage.users?.total ?? '—' }}</b>
          <span>acteurs</span>
        </div>
        <div class="kpi">
          <b>{{ data.openRegistrations ? 'Ouvertes' : 'Fermées' }}</b>
          <span>inscriptions</span>
        </div>
      </div>

      <dl class="dl">
        <div><dt>Version NodeInfo</dt><dd>{{ data.version }}</dd></div>
        <div><dt>Protocoles</dt><dd>{{ protocols.join(', ') || '—' }}</dd></div>
        <div v-if="metadata.nodeName"><dt>Nom du nœud</dt><dd>{{ metadata.nodeName }}</dd></div>
        <div v-if="metadata.purpose"><dt>Vocation</dt><dd>{{ metadata.purpose }}</dd></div>
      </dl>

      <h2>Document brut</h2>
      <RawJson :data="data" label="JSON NodeInfo" />
    </template>
  </div>
</template>
