import { createRouter, createWebHistory } from 'vue-router'

// 定义路由组件
const Home = () => import('../views/Home.vue')
const About = () => import('../views/About.vue')
const MusicSourceManager = () => import('../views/MusicSourceManager.vue')
const TrackDetail = () => import('../views/TrackDetail.vue')
const TestPage = () => import('../views/TestPage.vue')

// 定义路由
const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/about',
    name: 'About',
    component: About
  },
  {
    path: '/music-sources',
    name: 'MusicSourceManager',
    component: MusicSourceManager
  },
  {
    path: '/track/:trackId',
    name: 'TrackDetail',
    component: TrackDetail,
    props: true
  },
  {
    path: '/test',
    name: 'TestPage',
    component: TestPage
  }
]

// 创建路由实例
const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router