<script setup lang="ts">defineProps<{ name: string; size?: number }>()</script>
<template><svg :width="size ?? 17" :height="size ?? 17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
<template v-if="name === 'overview'"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></template>
<template v-else-if="name === 'editor'"><path d="m4 17 9-9 3 3-9 9H4v-3Z"/><path d="m14 7 2-2 3 3-2 2"/></template>
<template v-else-if="name === 'instances'"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 8h10M7 12h4M7 16h7"/></template>
<template v-else-if="name === 'exports'"><path d="M12 3v12m0 0 4-4m-4 4-4-4"/><path d="M5 19h14"/></template>
<template v-else-if="name === 'mods'"><path d="M8 3v3m8-3v3M3 8h3m12 0h3M3 16h3m12 0h3M8 18v3m8-3v3"/><rect x="6" y="6" width="12" height="12" rx="3"/></template>
<template v-else-if="name === 'changelog'"><path d="M6 3h9l3 3v15H6z"/><path d="M14 3v4h4M9 11h6M9 15h6"/></template>
<template v-else-if="name === 'logs'"><path d="m5 7 4 4-4 4M11 16h7"/><rect x="3" y="3" width="18" height="18" rx="2"/></template>
<template v-else-if="name === 'settings'"><circle cx="12" cy="12" r="3"/><path d="M19 15l2 2-4 4-2-2M9 5 7 3 3 7l2 2M12 2v4m0 12v4M2 12h4m12 0h4"/></template>
<template v-else-if="name === 'refresh'"><path d="M20 6v5h-5M4 18v-5h5"/><path d="M18.5 9A7 7 0 0 0 6.3 6.3L4 9m16 6-2.3 2.7A7 7 0 0 1 5.5 15"/></template>
<template v-else-if="name === 'search'"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></template><template v-else><circle cx="12" cy="12" r="8"/></template>
</svg></template>
