// Catch-all proxy from the browser to wilder's client API (`/api/v1/*`).
//
// The browser only ever talks to this Nuxt server (same BFF principle as the
// read-only pages). `proxyRequest` forwards the method, body and — crucially —
// the Cookie header to wilder, and relays wilder's Set-Cookie back, so the
// opaque session cookie round-trips transparently. No CORS, and the browser
// never reaches the ActivityPub server directly.
export default defineEventHandler(async (event) => {
  const { wilderUrl } = useRuntimeConfig(event)
  const path = (getRouterParam(event, 'path') || '').replace(/^\/+/, '')
  const search = getRequestURL(event).search
  const target = `${(wilderUrl as string).replace(/\/$/, '')}/api/v1/${path}${search}`
  return proxyRequest(event, target)
})
