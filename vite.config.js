import { run } from 'vite-plugin-run';

export default {
    "root": "./crates/server/assets",
    "server": {
      "headers": {
        "Cross-Origin-Opener-Policy": "same-origin",
        "Cross-Origin-Embedder-Policy": "require-corp"
      }
    },
    "plugins": [
      run([
        {
          name: 'build css',
          run: ['./scripts/build_css'],
          condition: (file) => file.includes('_input.css'),
        }
      ])
    ]
}
