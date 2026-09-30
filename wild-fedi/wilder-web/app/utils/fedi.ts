// Helpers de lecture du vocabulaire ActivityStreams / ActivityPub.
//
// Le fédiverse est permissif : un champ peut être une chaîne ou un tableau, un
// objet peut être imbriqué ou réduit à son IRI, le HTML des notes distantes est
// arbitraire. Ces fonctions absorbent cette variété pour que les composants
// restent simples — « soyez libéral dans ce que vous acceptez ».

export type AnyObject = Record<string, any>

/** Normalise un champ « un-ou-plusieurs » (to, cc, tag, attachment…) en tableau. */
export function asArray<T = any>(v: T | T[] | undefined | null): T[] {
  if (v == null) return []
  return Array.isArray(v) ? v : [v]
}

/** Extrait l'IRI d'une valeur, qu'elle soit une chaîne ou un objet { id }. */
export function iriOf(v: any): string {
  if (typeof v === 'string') return v
  if (v && typeof v === 'object') return v.id || v.href || ''
  return ''
}

/** L'hôte d'une URL, pour reconstruire un handle @nom@domaine. */
export function hostOf(url: string): string {
  try {
    return new URL(url).host
  } catch {
    return ''
  }
}

/** Le handle @nom@domaine d'un acteur, à partir de son document. */
export function handleOf(actor: AnyObject | null | undefined): string {
  if (!actor) return ''
  const name = actor.preferredUsername || (actor.id ? actor.id.split('/').pop() : '')
  const host = hostOf(actor.id || '')
  return name && host ? `@${name}@${host}` : name || ''
}

/** Libellé français lisible d'un type d'acteur. */
export function actorTypeLabel(type: string | undefined): string {
  switch (type) {
    case 'Person': return 'Personne'
    case 'Group': return 'Groupe · classe'
    case 'Organization': return 'Organisation · école'
    case 'Service': return 'Service · station de capture'
    case 'Application': return 'Application'
    default: return type || 'Acteur'
  }
}

/**
 * L'objet publié porté par une entrée d'outbox. Une observation arrive presque
 * toujours comme un `Create` enveloppant une `Note` ; on remonte à l'objet, mais
 * on retombe sur l'activité elle-même si l'objet n'est qu'un IRI (non résolu).
 */
export function objectOf(item: AnyObject): AnyObject {
  const obj = item?.object
  if (obj && typeof obj === 'object') return obj
  return item
}

/** Les hashtags (#castor…) d'un objet, via ses `tag` de type Hashtag. */
export function hashtagsOf(obj: AnyObject): string[] {
  return asArray(obj?.tag)
    .filter((t) => t && t.type === 'Hashtag' && t.name)
    .map((t) => String(t.name))
}

/** Les images jointes d'un objet, via ses `attachment` de type Image. */
export function imagesOf(obj: AnyObject): { url: string; name?: string }[] {
  return asArray(obj?.attachment)
    .filter((a) => a && /image/i.test(a.type || a.mediaType || ''))
    .map((a) => ({ url: iriOf(a.url) || iriOf(a.href), name: a.name }))
    .filter((a) => a.url)
}

/** Le lieu (`Place`) d'un objet, s'il porte des coordonnées. */
export function placeOf(obj: AnyObject): { name?: string; lat: number; lon: number } | null {
  const loc = obj?.location
  if (!loc || typeof loc !== 'object') return null
  const lat = Number(loc.latitude)
  const lon = Number(loc.longitude)
  if (Number.isNaN(lat) || Number.isNaN(lon)) return null
  return { name: loc.name, lat, lon }
}

/** Une activité/objet est-il adressé au public ? */
export function isPublic(item: AnyObject): boolean {
  const P = 'https://www.w3.org/ns/activitystreams#Public'
  return [...asArray(item?.to), ...asArray(item?.cc)].some((a) => iriOf(a) === P)
}

/**
 * Débarrasse un contenu de son HTML et le réduit à du texte. Les notes du
 * fédiverse contiennent du HTML arbitraire ; ce client de démonstration l'affiche
 * en texte plutôt que de l'injecter (pas de `v-html` : pas de vecteur XSS).
 */
export function toPlainText(html: string | undefined): string {
  if (!html) return ''
  return html
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&#39;|&apos;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

/** Date ISO → date française lisible (« 29 septembre 2026, 11:14 »). */
export function frDate(iso: string | undefined): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return new Intl.DateTimeFormat('fr-FR', {
    day: 'numeric', month: 'long', year: 'numeric',
    hour: '2-digit', minute: '2-digit'
  }).format(d)
}
