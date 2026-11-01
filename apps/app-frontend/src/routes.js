import { createRouter, createWebHistory } from 'vue-router'
const Pages = {
  Index: () => import('@/pages/Index.vue'),
  Browse: () => import('@/pages/Browse.vue'),
  Library: () => import('@/pages/Library.vue'),
  Settings: () => import('@/pages/Settings.vue'),
  Themes: () => import('@/pages/Themes.vue'),
  Assistant: () => import('@/pages/Assistant.vue'),
  MinecraftRust: () => import('@/pages/MinecraftRust.vue'),
}
const Project = {
  Index: () => import('@/pages/project/Index.vue'),
  Description: () => import('@/pages/project/Description.vue'),
  Versions: () => import('@/pages/project/Versions.vue'),
  Gallery: () => import('@/pages/project/Gallery.vue'),
  Version: () => import('@/pages/project/Version.vue'),
}
const Instance = {
  Index: () => import('@/pages/instance/Index.vue'),
  Mods: () => import('@/pages/instance/Mods.vue'),
  Options: () => import('@/pages/instance/Options.vue'),
  Logs: () => import('@/pages/instance/Logs.vue'),
  Health: () => import('@/pages/instance/Health.vue'),
}

/**
 * Configures application routing. Add page to pages/index and then add to route table here.
 */
export default createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/minecraft-rust',
      name: 'MinecraftRust',
      component: Pages.MinecraftRust,
      meta: { breadcrumb: [{ name: 'Minecraft Rust' }] },
    },
    {
      path: '/assistant',
      name: 'Assistant',
      component: Pages.Assistant,
      meta: { breadcrumb: [{ name: 'U3 AI' }] },
    },
    {
      path: '/',
      name: 'Home',
      component: Pages.Index,
      meta: {
        breadcrumb: [{ name: 'Home' }],
      },
    },
    {
      path: '/browse/:projectType',
      name: 'Browse',
      component: Pages.Browse,
      meta: {
        breadcrumb: [{ name: 'Browse' }],
      },
    },
    {
      path: '/library',
      name: 'Library',
      component: Pages.Library,
      meta: {
        breadcrumb: [{ name: 'Library' }],
      },
    },
    {
      path: '/themes',
      name: 'Themes',
      component: Pages.Themes,
      meta: { breadcrumb: [{ name: 'Темы' }] },
    },
    {
      path: '/settings',
      name: 'Settings',
      component: Pages.Settings,
      meta: {
        breadcrumb: [{ name: 'Settings' }],
      },
    },
    {
      path: '/project/:id',
      name: 'Project',
      component: Project.Index,
      props: true,
      children: [
        {
          path: '',
          name: 'Description',
          component: Project.Description,
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project' }],
          },
        },
        {
          path: 'versions',
          name: 'Versions',
          component: Project.Versions,
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Versions' }],
          },
        },
        {
          path: 'version/:version',
          name: 'Version',
          component: Project.Version,
          props: true,
          meta: {
            useContext: true,
            breadcrumb: [
              { name: '?Project', link: '/project/{id}/' },
              { name: 'Versions', link: '/project/{id}/versions' },
              { name: '?Version' },
            ],
          },
        },
        {
          path: 'gallery',
          name: 'Gallery',
          component: Project.Gallery,
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Gallery' }],
          },
        },
      ],
    },
    {
      path: '/instance/:id',
      name: 'Instance',
      component: Instance.Index,
      props: true,
      children: [
        {
          path: '',
          name: 'Mods',
          component: Instance.Mods,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance' }],
          },
        },
        {
          path: 'projects/:type',
          name: 'ModsFilter',
          component: Instance.Mods,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance' }],
          },
        },
        {
          path: 'options',
          name: 'Options',
          component: Instance.Options,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Options' }],
          },
        },
        {
          path: 'health',
          name: 'InstanceHealth',
          component: Instance.Health,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Diagnostics' }],
          },
        },
        {
          path: 'logs',
          name: 'Logs',
          component: Instance.Logs,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Logs' }],
          },
        },
      ],
    },
  ],
  linkActiveClass: 'router-link-active',
  linkExactActiveClass: 'router-link-exact-active',
  async scrollBehavior() {
    // This is an inner scroller, not the window. Wait for the routed component to mount.
    await new Promise((resolve) => requestAnimationFrame(resolve))
    document.querySelector('.router-view')?.scrollTo(0, 0)
    return false
  },
})
