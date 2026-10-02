// Proxy public media bytes through Nuxt. Media URLs minted by wilder use its
// federation domain (e.g. `wilder:8080` inside Docker), which a browser cannot
// resolve. The client rewrites those to a relative `/media/:id` (see
// `utils/media.ts`), and this route forwards them to the backend — same BFF
// principle as the API, extended to blobs.
export default defineEventHandler((event) => {
  const { wilderUrl } = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  return proxyRequest(event, `${(wilderUrl as string).replace(/\/$/, '')}/media/${id}`)
})
