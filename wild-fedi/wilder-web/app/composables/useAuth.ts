// Authentication state + actions, shared across the app via `useState`.
//
// `me` is fetched through `useRequestFetch` so it works during SSR too: that
// forwards the incoming request's cookies to our BFF, which forwards them to
// wilder. Mutations (login/logout/register) run from client handlers, where the
// browser attaches the session cookie automatically.

export type Role = 'school' | 'student' | 'service'

export interface Account {
  id: string
  username: string
  role: Role
  display_name: string | null
  email: string | null
  created_at: string
}

export interface NewAccountPayload {
  username: string
  password: string
  display_name?: string
  email?: string
}

export function useAuth() {
  const user = useState<Account | null>('auth-user', () => null)
  // Captured in setup/middleware context so it carries request cookies on SSR.
  const authFetch = useRequestFetch()

  /** Load the current account (null if not logged in). Safe to call repeatedly. */
  async function fetchMe(): Promise<Account | null> {
    try {
      user.value = await authFetch<Account>('/api/backend/auth/me')
    } catch {
      user.value = null
    }
    return user.value
  }

  async function login(username: string, password: string): Promise<Account> {
    user.value = await $fetch<Account>('/api/backend/auth/login', {
      method: 'POST',
      body: { username, password }
    })
    return user.value
  }

  async function registerSchool(payload: NewAccountPayload): Promise<Account> {
    user.value = await $fetch<Account>('/api/backend/auth/bootstrap', {
      method: 'POST',
      body: payload
    })
    return user.value
  }

  async function logout(): Promise<void> {
    await $fetch('/api/backend/auth/logout', { method: 'POST' })
    user.value = null
  }

  /** Is first-run school registration still open? */
  async function bootstrapAvailable(): Promise<boolean> {
    const r = await $fetch<{ available: boolean }>('/api/backend/auth/bootstrap')
    return r.available
  }

  const isSchool = computed(() => user.value?.role === 'school')

  return { user, isSchool, fetchMe, login, registerSchool, logout, bootstrapAvailable }
}
