// Accès serveur au fédiverse. Tout passe par ici : le navigateur n'émet jamais
// de requête vers un serveur ActivityPub, ce qui évite CORS et centralise
// l'en-tête `Accept` de négociation de contenu, les délais et les erreurs.

const AP_ACCEPT =
  'application/activity+json, application/ld+json; profile="https://www.w3.org/ns/activitystreams"'

/**
 * Récupère un document ActivityPub. Les serveurs répondent en
 * `application/activity+json` / `+jrd+json`, deux types que `ofetch` reconnaît
 * comme du JSON (motif `application/…+json`) et parse donc automatiquement.
 */
export function apFetch<T = any>(url: string): Promise<T> {
  assertHttpUrl(url)
  return $fetch<T>(url, {
    headers: { accept: AP_ACCEPT, 'user-agent': 'wilder-web (+client fédéré LICHEN)' },
    timeout: 10_000,
    // Un serveur distant renvoie parfois du JSON avec un type inattendu ;
    // on force l'interprétation JSON pour rester robuste.
    responseType: 'json'
  })
}

/**
 * Garde-fou : on ne suit que http(s). Le paramètre `url` venant du client, il
 * faudrait en production une liste blanche de domaines fédérés (anti-SSRF) ;
 * pour un client de démonstration local, on se limite au schéma.
 */
export function assertHttpUrl(url: string) {
  let u: URL
  try {
    u = new URL(url)
  } catch {
    throw createError({ statusCode: 400, statusMessage: `URL invalide : ${url}` })
  }
  if (u.protocol !== 'http:' && u.protocol !== 'https:') {
    throw createError({ statusCode: 400, statusMessage: 'Schéma non autorisé' })
  }
}

/** http en local (localhost / IP de bouclage / port), https ailleurs. */
export function baseFromDomain(domain: string): string {
  const local = /^(localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1\])(:\d+)?$/.test(domain)
  return `${local ? 'http' : 'https'}://${domain}`
}

/**
 * Base à utiliser pour joindre un domaine. Si c'est l'instance configurée, on
 * réutilise son URL complète, schéma compris. Indispensable en Docker : le
 * domaine y est un nom de service interne (`wilder:8080`) joignable en http, que
 * l'heuristique ci-dessus prendrait à tort pour du https. Pour un domaine tiers,
 * on retombe sur l'heuristique.
 */
export function baseForDomain(domain: string): string {
  try {
    const configured = String(useRuntimeConfig().wilderUrl || '')
    if (configured && new URL(configured).host === domain) {
      return configured.replace(/\/$/, '')
    }
  } catch {
    // Config indisponible : on ignore et on retombe sur l'heuristique.
  }
  return baseFromDomain(domain)
}

/**
 * Résout un handle `@nom@domaine` en IRI d'acteur via WebFinger (README §2) :
 * c'est l'étape qui fait ressembler les identités fédérées à des adresses e-mail.
 */
export async function resolveHandle(handle: string): Promise<string> {
  const clean = handle.trim().replace(/^@/, '')
  const [name, domain] = clean.split('@')
  if (!name || !domain) {
    throw createError({ statusCode: 400, statusMessage: 'Handle attendu : @nom@domaine' })
  }
  const base = baseForDomain(domain)
  const resource = encodeURIComponent(`acct:${clean}`)
  const jrd = await apFetch<any>(`${base}/.well-known/webfinger?resource=${resource}`)
  const self = asLinks(jrd).find(
    (l) => l.rel === 'self' && String(l.type || '').includes('activity+json')
  )
  if (!self?.href) {
    throw createError({ statusCode: 404, statusMessage: `Acteur introuvable : ${clean}` })
  }
  return self.href
}

function asLinks(jrd: any): { rel?: string; type?: string; href?: string }[] {
  return Array.isArray(jrd?.links) ? jrd.links : []
}
