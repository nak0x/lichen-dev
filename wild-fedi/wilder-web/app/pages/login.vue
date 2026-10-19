<script setup lang="ts">
// Login, and — on a fresh instance — first-run school registration. The server
// tells us which via the bootstrap endpoint, so the one page covers both.
definePageMeta({ layout: false })

const { login, registerSchool, bootstrapAvailable, user } = useAuth()
const router = useRouter()

const mode = ref<'login' | 'register'>('login')
const form = reactive({ username: '', password: '', display_name: '' })
const error = ref('')
const busy = ref(false)

onMounted(async () => {
  try {
    if (await bootstrapAvailable()) mode.value = 'register'
  } catch {
    // Server unreachable: stay on login and let the submit surface the error.
  }
})

async function submit() {
  error.value = ''
  busy.value = true
  try {
    if (mode.value === 'register') {
      await registerSchool({
        username: form.username,
        password: form.password,
        display_name: form.display_name || undefined
      })
    } else {
      await login(form.username, form.password)
    }
    router.push(user.value?.role === 'school' ? '/ecole' : '/moi')
  } catch (e: any) {
    error.value = e?.data?.error || 'Échec. Vérifiez vos identifiants.'
  } finally {
    busy.value = false
  }
}

useHead({ title: 'Wilder — connexion' })
</script>

<template>
  <div class="auth">
    <div class="auth-card card rise">
      <NuxtLink to="/" class="brand auth-brand">Wilder</NuxtLink>
      <Transition name="page" mode="out-in">
        <div :key="mode">
          <p class="oeil">{{ mode === 'register' ? 'Initialiser l’instance' : 'Connexion' }}</p>
          <h1>{{ mode === 'register' ? 'Créer l’école' : 'Se connecter' }}</h1>
          <p class="lede">
            {{ mode === 'register'
              ? 'Aucune école n’existe encore sur cette instance. Créez le compte qui l’administrera.'
              : 'Accédez à votre espace du réseau LICHEN.' }}
          </p>
        </div>
      </Transition>

      <form @submit.prevent="submit">
        <label>Identifiant
          <input v-model="form.username" type="text" autocapitalize="off" spellcheck="false" required />
        </label>
        <Transition name="list">
          <label v-if="mode === 'register'">Nom affiché
            <input v-model="form.display_name" type="text" placeholder="École du Lichen" />
          </label>
        </Transition>
        <label>Mot de passe
          <input v-model="form.password" type="password" required />
        </label>

        <Transition name="list">
          <p v-if="error" class="err">{{ error }}</p>
        </Transition>

        <button class="btn" type="submit" :class="{ loading: busy }" :disabled="busy">
          {{ mode === 'register' ? 'Créer l’école' : 'Se connecter' }}
        </button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.auth {
  min-height: 100svh; display: grid; place-items: center; padding: 2rem 1.25rem;
  background:
    radial-gradient(130% 80% at 50% -10%, var(--lichen-wash) 0%, transparent 60%),
    var(--bg);
}
.auth-card { width: 100%; max-width: 24rem; border-top: 3px solid var(--forest); }
.auth-brand {
  display: inline-block; font-family: var(--serif); font-weight: 600; font-size: .95rem;
  letter-spacing: .09em; text-transform: uppercase; color: var(--ink);
  text-decoration: none; margin-bottom: 1rem;
}
.auth-card h1 { margin: 0 0 .4rem; }
form { display: flex; flex-direction: column; gap: .9rem; margin-top: 1.2rem; }
label { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: var(--ink-soft); font-weight: 500; }
.btn { margin-top: .4rem; }
</style>
