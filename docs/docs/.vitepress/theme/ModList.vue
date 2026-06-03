<script setup>
import { ref, computed, onMounted } from "vue";

const props = defineProps({
  src: { type: String, required: true },
});

const groups = ref([]);
const loading = ref(true);
const error = ref(false);
const query = ref("");

function parseModlist(md) {
  const lines = md.split("\n");
  const result = [];
  let current = null;
  for (const raw of lines) {
    const line = raw.trim();
    const heading = line.match(/^##\s+(.*)$/);
    if (heading) {
      current = { title: heading[1].trim(), mods: [] };
      result.push(current);
      continue;
    }
    const mod = line.match(/^-\s+\[(.+)\]\((https?:\/\/[^)]+)\)\s*$/);
    if (mod && current) {
      current.mods.push({ name: mod[1].trim(), url: mod[2].trim() });
    }
  }
  return result.filter((g) => g.mods.length > 0);
}

const filteredGroups = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return groups.value;
  return groups.value
    .map((g) => ({
      title: g.title,
      mods: g.mods.filter((m) => m.name.toLowerCase().includes(q)),
    }))
    .filter((g) => g.mods.length > 0);
});

const totalCount = computed(() =>
  groups.value.reduce((n, g) => n + g.mods.length, 0)
);

onMounted(async () => {
  try {
    const res = await fetch(props.src);
    if (!res.ok) throw new Error("not ok");
    const text = await res.text();
    groups.value = parseModlist(text);
    loading.value = false;
  } catch (e) {
    error.value = true;
    loading.value = false;
  }
});
</script>

<template>
  <div class="modlist">
    <div v-if="loading" class="modlist-status">Loading mod list…</div>
    <div v-else-if="error" class="modlist-status modlist-error">
      Could not load the mod list.
    </div>
    <template v-else>
      <div class="modlist-toolbar">
        <input
          v-model="query"
          class="modlist-search"
          type="text"
          placeholder="Filter mods…"
          aria-label="Filter mods"
        />
        <span class="modlist-total">{{ totalCount }} mods</span>
      </div>

      <div v-if="filteredGroups.length === 0" class="modlist-status">
        No mods match "{{ query }}".
      </div>

      <div v-for="group in filteredGroups" :key="group.title" class="modlist-group">
        <h3 class="modlist-group-title">
          {{ group.title }}
          <span class="modlist-group-count">{{ group.mods.length }}</span>
        </h3>
        <ul class="modlist-items">
          <li v-for="mod in group.mods" :key="mod.url" class="modlist-item">
            <a :href="mod.url" target="_blank" rel="noopener noreferrer">{{ mod.name }}</a>
          </li>
        </ul>
      </div>
    </template>
  </div>
</template>

<style scoped>
.modlist {
  margin: 16px 0;
}

.modlist-status {
  padding: 16px;
  color: var(--vp-c-text-2);
  font-size: 14px;
}

.modlist-error {
  color: var(--vp-c-text-3);
}

.modlist-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.modlist-search {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background-color: var(--vp-c-bg-alt);
  color: var(--vp-c-text-1);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.modlist-search:focus {
  border-color: var(--vp-c-brand-1);
}

.modlist-total {
  font-size: 13px;
  color: var(--vp-c-text-3);
  white-space: nowrap;
}

.modlist-group {
  margin-bottom: 24px;
}

.modlist-group-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--vp-c-divider);
}

.modlist-group-count {
  font-size: 12px;
  font-weight: 500;
  color: var(--vp-c-brand-1);
  background-color: var(--vp-c-brand-soft);
  padding: 1px 8px;
  border-radius: 10px;
}

.modlist-items {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 4px 16px;
  list-style: none;
  padding: 0;
  margin: 0;
}

.modlist-item {
  padding: 0;
  margin: 0;
}

.modlist-item a {
  display: block;
  padding: 5px 0;
  color: var(--vp-c-text-1);
  text-decoration: none;
  font-size: 14px;
  transition: color 0.15s;
}

.modlist-item a:hover {
  color: var(--vp-c-brand-1);
}
</style>
