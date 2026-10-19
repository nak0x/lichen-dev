<script setup lang="ts">
// The determination key: a guided walk from the general to a species. Start at
// the roots, answer each discriminating question by choosing a child, follow the
// breadcrumb back if needed, and land on a species card.
definePageMeta({ middleware: 'auth' })

const authFetch = useRequestFetch()
const { data: roots } = await useAsyncData('det-roots', () =>
  authFetch<any[]>('/api/backend/determination')
)

// null = at the start (show roots); otherwise the current step.
const step = ref<any | null>(null)
const loading = ref(false)

async function go(id: string) {
  loading.value = true
  try {
    step.value = await $fetch<any>(`/api/backend/determination/${id}`)
  } finally {
    loading.value = false
  }
}

function restart() {
  step.value = null
}

const atSpecies = computed(() => step.value?.node?.is_species === true)
const choices = computed(() => (step.value ? step.value.children : roots.value || []))

useHead({ title: 'Clé de détermination — Wilder' })
</script>

<template>
  <div class="wrap">
    <p class="oeil"><Fa icon="key" /> Clé de détermination</p>
    <h1>Identifier une espèce</h1>
    <p class="lede">Du plus général au plus précis : répondez à chaque question.</p>

    <!-- Breadcrumb of choices so far -->
    <nav v-if="step" class="trail">
      <a href="#" @click.prevent="restart">Début</a>
      <template v-for="a in step.ancestors" :key="a.id">
        <span class="sep">›</span><a href="#" @click.prevent="go(a.id)">{{ a.name }}</a>
      </template>
      <span class="sep">›</span><span class="here">{{ step.node.name }}</span>
    </nav>

    <!-- Species reached -->
    <section v-if="atSpecies" class="species card">
      <p class="oeil"><Fa icon="leaf" /> Espèce</p>
      <h2>{{ step.node.vernacular || step.node.name }}</h2>
      <p v-if="step.node.vernacular" class="latin">{{ step.node.name }}</p>
      <p class="actions">
        <button class="btn" @click="restart">Recommencer</button>
      </p>
    </section>

    <!-- A question and its possible answers -->
    <section v-else>
      <p v-if="step?.node?.question" class="question">{{ step.node.question }}</p>
      <p v-else class="question">Par quoi commencer ?</p>

      <div v-if="loading" class="spin">…</div>
      <ul v-else class="choices">
        <li v-for="c in choices" :key="c.id">
          <button class="choice" @click="go(c.id)">
            <span class="name">{{ c.vernacular || c.name }}</span>
            <span v-if="c.rank" class="rank">{{ c.rank }}</span>
          </button>
        </li>
      </ul>
      <p v-if="step" class="actions">
        <button class="btn ghost sm" @click="restart">Tout recommencer</button>
      </p>
    </section>
  </div>
</template>

<style scoped>
.trail { font-size: .85rem; color: var(--ink-soft); margin: 1rem 0; display: flex; flex-wrap: wrap; gap: .3rem; align-items: center; }
.trail .sep { color: var(--line-strong); }
.trail .here { color: var(--forest); font-weight: 600; }
.question { font-family: var(--serif); font-size: 1.25rem; color: var(--forest-ink); margin: 1.2rem 0 1rem; }
.choices { list-style: none; padding: 0; display: grid; gap: .6rem; grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr)); }
.choice {
  width: 100%; text-align: left; font: inherit; cursor: pointer;
  background: var(--paper); border: 1px solid var(--line); border-left: 2px solid var(--lichen);
  border-radius: var(--radius); padding: .8rem 1rem;
}
.choice:hover { border-color: var(--forest); border-left-color: var(--forest); background: var(--lichen-wash); }
.choice .name { display: block; font-weight: 600; }
.choice .rank { display: block; font-size: .75rem; color: var(--ink-soft); }
.species { border-left: 2px solid var(--forest); }
.species .latin { font-style: italic; color: var(--ink-soft); margin-top: -.3rem; }
</style>
