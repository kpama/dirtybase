import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Dirtybase",
  description: "Dirtybase framework and documentation site",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Home", link: "/" },
      { text: "Docs", link: "/docs/v1/installation" },
    ],

    sidebar: [
      {
        items: [
          {
            text: "Getting Started",
            items: [
              { text: "Installation", link: "/docs/v1/installation" },
              { text: "Configuration", link: "/docs/v1/configuration" },
              { text: "env variables", link: "/docs/v1/config/env_config" },
            ],
            collapsed: true,
          },
          {
            text: "The Basics",
            items: [{ text: "Routing", link: "/docs/v1/routing" }],
            collapsed: true,
          },
          {
            text: "Database",
            items: [
              { text: "Migrations", link: "/docs/v1/migrations" },
              { text: "Seeding", link: "/docs/v1/seeding" },
            ],
            collapsed: true,
          },
          {
            text: "ORM",
            items: [
              { text: "Getting Started", link: "/docs/v1/orm/index" },
              { text: "Model", link: "/docs/v1/orm/model" },
              { text: "Repository", link: "/docs/v1/orm/Repository" },
              { text: "Relation", link: "/docs/v1/orm/relationship" },
              { text: "Deeper", link: "/docs/v1/orm/deeper.md" },
            ],
            collapsed: true,
          },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/shiftrightonce/dirtybase" },
    ],
  },
});
