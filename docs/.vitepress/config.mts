import { defineConfig } from 'vitepress'

const nav = [
  { text: 'Home', link: '/' },
  { text: 'Playground', link: '/playground' }
]

const sidebar = [
  {
    text: 'Introduction',
    items: [
      { text: 'What is Zap?', link: '/intro/what-is-zap' },
      { text: 'Getting Started', link: '/intro/getting-started' },
    ]
  },
  {
    text: 'Configuring Zap',
    items: [
      { text: 'Options', link: '/config/options' },
      { text: 'Types', link: '/config/types' },
      { text: 'Events', link: '/config/events' },
      { text: 'Functions', link: '/config/functions' },
    ]
  },
  {
    text: 'Using Zap',
    items: [
      { text: 'Generate Code', link: '/usage/generation' },
      { text: 'Event Usage', link: '/usage/events' },
      { text: 'Tooling Intergration', link: '/usage/tooling' },
    ]
  }
]

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Zap",
  description: "A lightning fast, type-safe, and easy to use networking solution for Roblox.",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav,
    sidebar,
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/red-blox/zap' },
      { icon: 'discord', link: 'https://discord.gg/mchCdAFPWU' },
    ]
  },
  head: [
    ['link', { rel: 'icon', href: '/logo.svg' }]
  ],
  vite: {
    configFile: "./docs/.vitepress/vite.config.ts"
  },
})
