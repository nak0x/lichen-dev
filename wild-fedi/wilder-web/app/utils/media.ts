// Rewrite a backend media URL to a Nuxt-relative path so the browser loads it
// through our media proxy (`server/routes/media/[id].ts`) rather than hitting
// the backend's internal domain directly. Any `.../media/<id>` becomes
// `/media/<id>`; anything else is returned unchanged.
export function mediaSrc(url: string | null | undefined): string {
  if (!url) return ''
  const m = url.match(/\/media\/([^/?#]+)/)
  return m ? `/media/${m[1]}` : url
}
