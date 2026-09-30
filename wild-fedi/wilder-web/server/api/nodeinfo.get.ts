// GET /api/nodeinfo?base=<url d'instance>
// Découvre puis renvoie le document NodeInfo 2.1 (README §4, contrat 7) : ce que
// l'instance déclare faire tourner. Base par défaut : le serveur wilder configuré.
export default defineEventHandler(async (event) => {
  const q = getQuery(event)
  const base = (typeof q.base === 'string' && q.base) || (useRuntimeConfig(event).wilderUrl as string)

  const discovery = await apFetch<any>(`${base.replace(/\/$/, '')}/.well-known/nodeinfo`)
  const link = (discovery.links || []).find((l: any) => String(l.rel || '').includes('nodeinfo'))
  if (!link?.href) {
    throw createError({ statusCode: 404, statusMessage: 'NodeInfo indisponible sur cette instance' })
  }
  return await apFetch(link.href)
})
