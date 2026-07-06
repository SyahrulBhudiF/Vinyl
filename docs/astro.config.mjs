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
          label: 'Start',
          items: [
            { label: 'Home', link: '/' },
            { label: 'Install', slug: 'install' },
            { label: 'Editor Setup', slug: 'editor-setup' },
            { label: 'Project Layout', slug: 'project-layout' },
          ],
        },
        {
          label: 'Authoring',
          items: [
            { label: 'Script Language', slug: 'script-language' },
            { label: 'Localization', slug: 'localization' },
            { label: 'Assets', slug: 'assets' },
          ],
        },
      ],
    }),
  ],
});
