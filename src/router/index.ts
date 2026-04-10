import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/splash',
      name: 'splash',
      component: () => import('@/views/SplashView.vue'),
    },
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/HomeView.vue'),
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/splash'
    }
  ],
})

export default router