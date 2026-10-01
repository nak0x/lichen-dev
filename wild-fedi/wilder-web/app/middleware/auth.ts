// Route guard for the authenticated app. Pages opt in with
// `definePageMeta({ middleware: 'auth' })`. Redirects to /login when there is no
// valid session.
export default defineNuxtRouteMiddleware(async () => {
  const { user, fetchMe } = useAuth()
  if (!user.value) {
    await fetchMe()
  }
  if (!user.value) {
    return navigateTo('/login')
  }
})
