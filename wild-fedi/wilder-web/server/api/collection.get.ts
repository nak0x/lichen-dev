// GET /api/collection?url=<iri d'une OrderedCollection>
// Sert l'outbox et la collection de followers. On renvoie une forme stable
// { totalItems, items } et on suit une éventuelle première page — un serveur
// complet pagine ces collections (OrderedCollectionPage).
export default defineEventHandler(async (event) => {
  const q = getQuery(event)
  const url = typeof q.url === 'string' ? q.url : ''
  if (!url) {
    throw createError({ statusCode: 400, statusMessage: 'Paramètre url requis' })
  }

  const col = await apFetch<any>(url)
  let items: any[] = col.orderedItems || col.items || []

  // Cas paginé : la collection ne contient pas les items mais pointe une page.
  if (items.length === 0 && col.first) {
    const firstUrl = typeof col.first === 'string' ? col.first : col.first.id
    if (firstUrl) {
      const page = await apFetch<any>(firstUrl)
      items = page.orderedItems || page.items || []
    }
  }

  return { totalItems: col.totalItems ?? items.length, items }
})
