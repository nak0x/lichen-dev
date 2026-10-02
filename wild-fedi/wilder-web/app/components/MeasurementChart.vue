<script setup lang="ts">
// One line chart for a single metric's time series. Client-only (canvas), one
// chart per metric so differing units/scales (°C vs pH) never share an axis. A
// category x-axis of formatted times avoids pulling a date-adapter dependency.
import { Chart } from 'chart.js/auto'

const props = defineProps<{ series: { metric: string; unit: string | null; points: { t: string; v: number }[] } }>()
const canvas = ref<HTMLCanvasElement | null>(null)
let chart: Chart | null = null

function label(iso: string) {
  return new Date(iso).toLocaleString('fr-FR', {
    day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit'
  })
}

onMounted(() => {
  if (!canvas.value) return
  const unit = props.series.unit ? ` (${props.series.unit})` : ''
  chart = new Chart(canvas.value, {
    type: 'line',
    data: {
      labels: props.series.points.map((p) => label(p.t)),
      datasets: [
        {
          label: props.series.metric + unit,
          data: props.series.points.map((p) => p.v),
          borderColor: '#2f5540',
          backgroundColor: 'rgba(47,85,64,.12)',
          tension: 0.25,
          fill: true,
          pointRadius: 3
        }
      ]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { display: true } },
      scales: { y: { beginAtZero: false } }
    }
  })
})

onBeforeUnmount(() => chart?.destroy())
</script>

<template>
  <div class="chart"><canvas ref="canvas" /></div>
</template>

<style scoped>
.chart { position: relative; height: 16rem; }
</style>
