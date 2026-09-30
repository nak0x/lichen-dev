// GET /api/actor?h=@nom@domaine   ou   ?url=<iri d'acteur>
// Résout (WebFinger si besoin) puis renvoie le document d'acteur tel quel : les
// composants savent le lire défensivement, pour un acteur local comme distant.
export default defineEventHandler(async (event) => {
  const q = getQuery(event)
  let url = typeof q.url === 'string' ? q.url : ''

  if (!url && typeof q.h === 'string' && q.h) {
    url = await resolveHandle(q.h)
  }
  if (!url) {
    // Défaut : l'acteur d'accueil configuré (l'acteur de démo de wilder).
    const home = useRuntimeConfig(event).public.homeHandle as string
    url = await resolveHandle(home)
  }

  return await apFetch(url)
})
