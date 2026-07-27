/**
 * Teaches `bun test` to compile single-file components.
 *
 * Bun has no built-in `.vue` loader, so without this an `import Foo from
 * './Foo.vue'` silently resolves to a stand-in and every assertion against the
 * rendered markup passes while testing nothing. That failure mode is worse
 * than having no component tests, which is why this exists.
 *
 * Only behaviour is compiled. Styles are dropped — `<style scoped>` affects
 * appearance, which these tests do not assert, and compiling it would pull a
 * PostCSS pipeline into the test run for nothing.
 */

import { createHash } from 'node:crypto'

import { compileScript, parse } from '@vue/compiler-sfc'
import { plugin } from 'bun'

plugin({
  name: 'vue-sfc',
  setup(build) {
    build.onLoad({ filter: /\.vue$/ }, async (args) => {
      const source = await Bun.file(args.path).text()
      const { descriptor, errors } = parse(source, { filename: args.path })
      if (errors.length) {
        throw new Error(`Failed to parse ${args.path}: ${errors.map(String).join(', ')}`)
      }

      // Vue derives scoped-style ids and hot-reload keys from this; any stable
      // per-file value works, and the path hash keeps it deterministic.
      const id = createHash('sha256').update(args.path).digest('hex').slice(0, 8)

      // `inlineTemplate` folds the render function into the setup function,
      // which avoids emitting a second module and keeps bindings in scope.
      const compiled = compileScript(descriptor, { id, inlineTemplate: true })

      return { contents: compiled.content, loader: 'ts' }
    })
  },
})
