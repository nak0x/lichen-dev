<script setup lang="ts">
/**
 * Fond animé de la page d'accueil : des thalles de lichen qui dérivent
 * lentement, et une poussière de spores.
 *
 * Rien n'est chargé : les formes sont dessinées à l'exécution (une fois, dans
 * des canevas hors écran) puis simplement déplacées — pas d'image, pas de
 * requête, et une boucle qui s'arrête dès que la page n'est plus visible.
 * Si le système demande moins d'animation, une seule image fixe est rendue.
 */

const props = withDefaults(defineProps<{ thalles?: number; spores?: number }>(), {
  thalles: 9,
  spores: 70
})

const cible = ref<HTMLCanvasElement | null>(null)

type Thalle = { img: HTMLCanvasElement; t: number; x: number; y: number; r: number; vr: number; vx: number; vy: number; a: number; p: number }
type Spore = { x: number; y: number; vx: number; vy: number; d: number; a: number; ph: number; w: number }

onMounted(() => {
  const cv = cible.value
  if (!cv) return
  const g = cv.getContext('2d', { alpha: true })
  if (!g) return

  const doux = window.matchMedia('(prefers-reduced-motion: reduce)')
  const sombre = window.matchMedia('(prefers-color-scheme: dark)')
  const dpr = Math.min(window.devicePixelRatio || 1, 2)

  let L = 0
  let H = 0
  let thalles: Thalle[] = []
  let spores: Spore[] = []
  let raf = 0
  let dernier = 0
  let visible = true

  // Parallaxe : la souris déplace à peine le décor, jamais le texte.
  const souris = { x: 0, y: 0, cx: 0, cy: 0 }

  const alea = (a: number, b: number) => a + Math.random() * (b - a)

  /** Les couleurs viennent des variables CSS : le fond suit le thème. */
  function teinte(nom: string, alpha: number) {
    const v = getComputedStyle(cv!).getPropertyValue(nom).trim() || '#8ba585'
    const h = v.replace('#', '')
    const n = h.length === 3 ? h.split('').map((c) => c + c).join('') : h
    const i = parseInt(n, 16)
    return `rgba(${(i >> 16) & 255}, ${(i >> 8) & 255}, ${i & 255}, ${alpha})`
  }

  /**
   * Un thalle : un contour lobé (somme d'harmoniques sur le rayon), quelques
   * lignes de croissance concentriques, et des apothécies.
   */
  function thalle(taille: number) {
    const s = Math.max(8, Math.round(taille * dpr))
    const c = document.createElement('canvas')
    c.width = c.height = s
    const x = c.getContext('2d')!
    x.translate(s / 2, s / 2)

    const R = (s / 2) * 0.88
    const ondes = [
      { k: 3, a: alea(0.05, 0.13), p: alea(0, 6.283) },
      { k: 5, a: alea(0.04, 0.09), p: alea(0, 6.283) },
      { k: 9, a: alea(0.02, 0.05), p: alea(0, 6.283) },
      { k: 17, a: alea(0.008, 0.022), p: alea(0, 6.283) }
    ]
    const rayon = (t: number) => {
      let r = 1
      for (const o of ondes) r += o.a * Math.sin(o.k * t + o.p)
      return R * r
    }
    const contour = (e: number) => {
      const p = new Path2D()
      const N = 240
      for (let i = 0; i <= N; i++) {
        const t = (i / N) * 6.283185
        const r = rayon(t) * e
        const px = Math.cos(t) * r
        const py = Math.sin(t) * r
        i ? p.lineTo(px, py) : p.moveTo(px, py)
      }
      p.closePath()
      return p
    }

    x.fillStyle = teinte('--lichen', 0.5)
    x.fill(contour(1))
    x.lineWidth = Math.max(1, s * 0.005)
    x.strokeStyle = teinte('--forest', 0.55)
    x.stroke(contour(1))
    x.strokeStyle = teinte('--forest', 0.22)
    for (const e of [0.78, 0.56, 0.34]) x.stroke(contour(e))

    x.fillStyle = teinte('--forest', 0.42)
    const n = Math.round(alea(3, 8))
    for (let i = 0; i < n; i++) {
      const t = alea(0, 6.283)
      const r = rayon(t) * alea(0.12, 0.66)
      x.beginPath()
      x.arc(Math.cos(t) * r, Math.sin(t) * r, s * alea(0.011, 0.028), 0, 6.284)
      x.fill()
    }
    return c
  }

  function peupler() {
    const aire = (L * H) / (1280 * 720)
    const nT = Math.max(4, Math.round(props.thalles * Math.min(1.4, Math.max(0.5, aire))))
    const nS = Math.max(20, Math.round(props.spores * Math.min(1.5, Math.max(0.4, aire))))

    thalles = Array.from({ length: nT }, () => {
      const t = alea(70, 230)
      return {
        img: thalle(t),
        t,
        x: alea(0, L),
        y: alea(0, H),
        r: alea(0, 6.283),
        vr: alea(-0.012, 0.012),
        vx: alea(-3.2, 3.2),
        vy: alea(-2.2, 2.2),
        a: alea(0.1, 0.26),
        p: alea(0.25, 1)
      }
    })

    spores = Array.from({ length: nS }, () => ({
      x: alea(0, L),
      y: alea(0, H),
      vx: alea(-5, 5),
      vy: alea(-13, -3),
      d: alea(0.5, 1.9),
      a: alea(0.16, 0.5),
      ph: alea(0, 6.283),
      w: alea(0.3, 1.1)
    }))
  }

  function redimensionner() {
    const p = cv!.parentElement
    L = p ? p.clientWidth : window.innerWidth
    H = p ? p.clientHeight : window.innerHeight
    cv!.width = Math.round(L * dpr)
    cv!.height = Math.round(H * dpr)
    cv!.style.width = L + 'px'
    cv!.style.height = H + 'px'
    g!.setTransform(dpr, 0, 0, dpr, 0, 0)
  }

  function dessiner(dt: number, temps: number) {
    g!.clearRect(0, 0, L, H)

    souris.cx += (souris.x - souris.cx) * 0.045
    souris.cy += (souris.y - souris.cy) * 0.045

    for (const t of thalles) {
      t.x += t.vx * dt
      t.y += t.vy * dt
      t.r += t.vr * dt
      const m = t.t
      if (t.x < -m) t.x = L + m
      if (t.x > L + m) t.x = -m
      if (t.y < -m) t.y = H + m
      if (t.y > H + m) t.y = -m

      g!.save()
      g!.globalAlpha = t.a
      g!.translate(t.x + souris.cx * 26 * t.p, t.y + souris.cy * 18 * t.p)
      g!.rotate(t.r)
      g!.drawImage(t.img, -m / 2, -m / 2, m, m)
      g!.restore()
    }

    g!.fillStyle = teinte('--forest', 1)
    for (const s of spores) {
      s.x += (s.vx + Math.sin(temps * 0.5 + s.ph) * 4) * dt
      s.y += s.vy * dt
      if (s.y < -6) {
        s.y = H + 6
        s.x = alea(0, L)
      }
      if (s.x < -6) s.x = L + 6
      if (s.x > L + 6) s.x = -6
      g!.globalAlpha = s.a * (0.55 + 0.45 * Math.sin(temps * s.w + s.ph))
      g!.beginPath()
      g!.arc(s.x + souris.cx * 40, s.y + souris.cy * 28, s.d, 0, 6.284)
      g!.fill()
    }
    g!.globalAlpha = 1
  }

  function boucle(ms: number) {
    if (!visible) return
    const dt = Math.min(0.05, (ms - dernier) / 1000) || 0
    dernier = ms
    dessiner(dt, ms / 1000)
    raf = requestAnimationFrame(boucle)
  }

  function jouer() {
    cancelAnimationFrame(raf)
    if (doux.matches) {
      dessiner(0, 0)
      return
    }
    visible = true
    dernier = performance.now()
    raf = requestAnimationFrame(boucle)
  }

  function suspendre() {
    visible = false
    cancelAnimationFrame(raf)
  }

  const surSouris = (e: PointerEvent) => {
    souris.x = (e.clientX / window.innerWidth) * 2 - 1
    souris.y = (e.clientY / window.innerHeight) * 2 - 1
  }
  const surVisibilite = () => (document.hidden ? suspendre() : jouer())
  const surTheme = () => {
    peupler()
    if (doux.matches) dessiner(0, 0)
  }

  let attente = 0
  const ro = new ResizeObserver(() => {
    redimensionner()
    clearTimeout(attente)
    // Le nombre de formes dépend de l'aire : on ne le recalcule qu'une fois
    // le redimensionnement terminé, jamais pendant.
    attente = window.setTimeout(() => {
      peupler()
      if (doux.matches) dessiner(0, 0)
    }, 220)
  })
  if (cv.parentElement) ro.observe(cv.parentElement)

  // La boucle ne tourne pas quand le fond est sorti de l'écran.
  const io = new IntersectionObserver((e) => (e[0]?.isIntersecting ? jouer() : suspendre()), { threshold: 0 })
  io.observe(cv)

  redimensionner()
  peupler()
  jouer()

  window.addEventListener('pointermove', surSouris, { passive: true })
  document.addEventListener('visibilitychange', surVisibilite)
  sombre.addEventListener('change', surTheme)
  doux.addEventListener('change', jouer)

  onBeforeUnmount(() => {
    suspendre()
    clearTimeout(attente)
    ro.disconnect()
    io.disconnect()
    window.removeEventListener('pointermove', surSouris)
    document.removeEventListener('visibilitychange', surVisibilite)
    sombre.removeEventListener('change', surTheme)
    doux.removeEventListener('change', jouer)
  })
})
</script>

<template>
  <canvas ref="cible" class="fond" aria-hidden="true" />
</template>

<style scoped>
.fond {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>
