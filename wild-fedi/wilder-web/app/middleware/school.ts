// Route guard for school-only areas (instance administration). Runs after the
// `auth` middleware; sends non-schools back to their personal space.
export default defineNuxtRouteMiddleware(async () => {
  const { user, fetchMe } = useAuth()
  if (!user.value) {
    await fetchMe()
  }
  if (!user.value) {
    return navigateTo('/login')
  }
  if (user.value.role !== 'school') {
    return navigateTo('/moi')
  }
})
