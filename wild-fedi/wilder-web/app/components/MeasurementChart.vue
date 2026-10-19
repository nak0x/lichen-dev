<script setup lang="ts">
// One line chart for a single metric's time series. Client-only (canvas), one
// chart per metric so differing units/scales (°C vs pH) never share an axis. A
// category x-axis of formatted times avoids pulling a date-adapter dependency.
// Colours are read from the theme's CSS variables so it follows light/dark.
import { Chart } from 'chart.js/auto'

const props = defineProps<{ series: { metric: string; unit: string | null; points: { t: string; v: number }[] } }>()
const canvas = ref<HTMLCanvasElement | null>(null)
const { theme } = useTheme()
let chart: Chart | null = null

function label(iso: string) {
  return new Date(iso).toLocaleString('fr-FR', {
    day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit'
  })
}

// Resolve a CSS custom property to a concrete colour (Chart.js needs real values).
function cssVar(name: string, fallback: string) {
  if (!canvas.value) return fallback
  return getComputedStyle(canvas.value).getPropertyValue(name).trim() || fallback
}

function applyTheme() {
  if (!chart) return
  const primary = cssVar('--primary', '#3a6f4b')
  const ink = cssVar('--ink-soft', '#586a59')
  const line = cssVar('--line', '#d6e0ce')
  const ds = chart.data.datasets[0] as any
  ds.borderColor = primary
  ds.backgroundColor = `color-mix(in srgb, ${primary} 16%, transparent)`
  ds.pointBackgroundColor = primary
  chart.options.plugins!.legend!.labels = { color: ink }
  const scaleColor = (s: any) => { s.ticks = { color: ink }; s.grid = { color: line } }
  scaleColor(chart.options.scales!.x)
  scaleColor(chart.options.scales!.y)
  chart.update()
}

onMounted(() => {
  if (!canvas.value) return
  const unit = props.series.unit ? ` (${props.series.unit})` : ''
  chart = new Chart(canvas.value, {
    type: 'line',
    data: {
      labels: props.series.points.map((p) => label(p.t)),
      datasets: [{
        label: props.series.metric + unit,
        data: props.series.points.map((p) => p.v),
        tension: 0.3,
        fill: true,
        pointRadius: 3,
        borderWidth: 2
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { display: true } },
      scales: { x: {}, y: { beginAtZero: false } }
    }
  })
  applyTheme()
})

// Re-tint when the user switches theme.
watch(theme, () => nextTick(applyTheme))

onBeforeUnmount(() => chart?.destroy())
</script>

<template>
  <div class="chart"><canvas ref="canvas" /></div>
</template>

<style scoped>
.chart { position: relative; height: 16rem; background: var(--surface); border: 1px solid var(--line); border-radius: var(--r-lg); padding: .8rem; box-shadow: var(--shadow); }
</style>
