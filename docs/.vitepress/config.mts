import { defineConfig } from 'vitepress'
import { tabsMarkdownPlugin } from 'vitepress-plugin-tabs'

const nav = [
  { text: 'Home', link: '/' },
  { text: 'Playground', link: '/playground' }
]

const sidebar = [
  {
    text: 'Getting Started',
    items: [
      { text: 'Installation', link: '/install' },
    ]
  },
  {
    text: 'Configuring Zap',
    items: [
      { text: 'Introduction', link: '/config/intro' },
      { text: 'Options', link: '/config/options' },
      { text: 'Types', link: '/config/types' },
      { text: 'Events', link: '/config/events' },
    ]
  },
  {
    text: 'Using Zap',
    items: [
      { text: 'Generate Code', link: '/usage/generation' },
      { text: 'Event Usage', link: '/usage/events' },
    ]
  }
]

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Zap",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav,
    sidebar,
    socialLinks: [
      { icon: 'github', link: 'https://github.com/red-blox/zap' },
      { icon: 'discord', link: 'https://discord.gg/mchCdAFPWU' },
    ]
  },
  markdown: {
    config(md) {
      md.use(tabsMarkdownPlugin)
    }
  },
  vite: {
    configFile: "./docs/.vitepress/vite.config.ts"
  }
})