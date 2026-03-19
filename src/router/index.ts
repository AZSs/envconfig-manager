import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/config'
    },
    {
      path: '/config',
      name: 'ConfigEditor',
      component: () => import('../views/ConfigEditor.vue')
    },
    {
      path: '/env',
      name: 'EnvVariables',
      component: () => import('../views/EnvVariables.vue')
    },
    {
      path: '/backup',
      name: 'BackupManager',
      component: () => import('../views/BackupManager.vue')
    },
    {
      path: '/profiles',
      name: 'ProfileManager',
      component: () => import('../views/ProfileManager.vue')
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('../views/Settings.vue')
    }
  ]
})

export default router
