import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  integrations: [
    starlight({
      title: 'Vinyl',
      customCss: ['./src/styles/vinyl.css'],
      sidebar: [
        {
          label: 'Start',
          items: [
            { label: 'Install', slug: 'install' },
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
