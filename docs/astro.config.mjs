import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

const vnGrammar = JSON.parse(
  readFileSync(
    fileURLToPath(new URL('../editors/vscode-vinyl/syntaxes/vinyl-vn.tmLanguage.json', import.meta.url)),
    'utf8',
  ),
);

export default defineConfig({
  site: 'https://syahrulbhudif.github.io',
  base: '/Vinyl',
  integrations: [
    starlight({
      title: 'Vinyl',
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/SyahrulBhudiF/Vinyl',
        },
      ],
      customCss: ['./src/styles/vinyl.css'],
      expressiveCode: {
        shiki: {
          langs: [vnGrammar],
        },
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Home', link: '/' },
            { label: 'Install', slug: 'install' },
            { label: 'Quickstart', slug: 'quickstart' },
            { label: 'Editor Setup', slug: 'editor-setup' },
          ],
        },
        {
          label: 'Game Authoring',
          items: [
            { label: 'Project Layout', slug: 'project-layout' },
            { label: 'Script Language', slug: 'script-language' },
            { label: 'Assets', slug: 'assets' },
            { label: 'Localization', slug: 'localization' },
          ],
        },
        {
          label: 'Player',
          items: [
            { label: 'Controls and Settings', slug: 'player' },
            { label: 'Saves and Rollback', slug: 'saves' },
            { label: 'CLI Reference', slug: 'cli' },
            { label: 'Troubleshooting', slug: 'troubleshooting' },
          ],
        },
        {
          label: 'Engine',
          items: [
            { label: 'Architecture', slug: 'architecture' },
            { label: 'Development and Releases', slug: 'development' },
            { label: 'Performance', slug: 'performance' },
          ],
        },
      ],
    }),
  ],
});
