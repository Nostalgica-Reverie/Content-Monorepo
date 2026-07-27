<script setup lang="ts">
/**
 * Visual datapack generators.
 *
 * The forms are not written here — they are drawn from mcdoc schemas by
 * `McdocField`, the same way upstream's generators work. This page only picks
 * a schema, owns the document being edited, and shows the JSON that comes out.
 */
import { computed, ref, watch } from 'vue'

import Button from '@/components/ui/Button.vue'
import McdocField from '@/mcdoc/McdocField.vue'
import { fixtureSchemaSource, generatorDefinitions } from '@/mcdoc/fixtures'
import { rootPath } from '@/mcdoc/simplify'
import { defaultValue } from '@/mcdoc/value'
import { useShellStore } from '@/stores/shell'
import { useToastsStore } from '@/stores/toasts'

const shell = useShellStore()
const toasts = useToastsStore()
const source = fixtureSchemaSource
const selectedId = ref(generatorDefinitions[0]?.id ?? '')
const document = ref<unknown>({})

const generator = computed(
  () => generatorDefinitions.find((entry) => entry.id === selectedId.value) ?? generatorDefinitions[0],
)

const json = computed(() => {
  try {
    return JSON.stringify(document.value, null, 2)
  } catch {
    return ''
  }
})

function reset() {
  document.value = generator.value ? defaultValue(generator.value.type, source) : {}
}

async function copy() {
  await navigator.clipboard.writeText(json.value)
  toasts.push('Copied', 'Generated JSON copied to the clipboard.', 'success')
}

watch(selectedId, reset, { immediate: true })

/** Honour the generator an extension asked for, including on first mount. */
watch(
  () => shell.generatorRequest,
  (request) => {
    if (request.id && generatorDefinitions.some((entry) => entry.id === request.id)) {
      selectedId.value = request.id
    }
  },
  { immediate: true, deep: true },
)
</script>

<template>
  <section class="grid view-grid">
    <div class="panel span-7">
      <div class="panel-head">
        <div>
          <h2>Generators</h2>
          <p class="panel-copy">
            Forms are drawn from mcdoc schemas, so every registry the schemas describe is editable here.
          </p>
        </div>
        <div class="panel-actions">
          <select v-model="selectedId" class="generator-picker">
            <option v-for="entry in generatorDefinitions" :key="entry.id" :value="entry.id">
              {{ entry.title }}
            </option>
          </select>
          <Button variant="quiet" @click="reset">Reset</Button>
        </div>
      </div>
      <div class="generator-form">
        <McdocField
          v-if="generator"
          :type="generator.type"
          :path="rootPath(document)"
          :source="source"
          @update="document = $event"
        />
      </div>
    </div>

    <div class="panel span-5">
      <div class="panel-head">
        <div>
          <h2>Output</h2>
          <p class="panel-copy">{{ generator?.folder }}</p>
        </div>
        <div class="panel-actions"><Button variant="quiet" @click="copy">Copy</Button></div>
      </div>
      <pre class="generator-output">{{ json }}</pre>
    </div>
  </section>
</template>

<style scoped>
.generator-form {
  overflow: auto;
  padding: 12px 4px 12px 0;
}

.generator-output {
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.5;
  margin: 0;
  overflow: auto;
  padding: 12px 0;
  white-space: pre;
}

.generator-picker {
  background: var(--surface-2);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
  padding: 5px 8px;
}
</style>
