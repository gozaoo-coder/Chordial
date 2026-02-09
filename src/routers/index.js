import { createRouter, createWebHistory } from 'vue-router'

// 定义路由组件
const Home = () => import('../views/Home.vue')
const About = () => import('../views/About.vue')
const Tracks = () => import('../views/Tracks.vue')
const Artists = () => import('../views/Artists.vue')
const Albums = () => import('../views/Albums.vue')
const MusicSourceManager = () => import('../views/MusicSourceManager.vue')
const TrackDetail = () => import('../views/TrackDetail.vue')
const PlayerView = () => import('../views/PlayerView.vue')
const SettingsView = () => import('../views/SettingsView.vue')
const TestPage = () => import('../views/TestPage.vue')

// 定义路由
const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    meta: { title: '首页' }
  },
  {
    path: '/tracks',
    name: 'Tracks',
    component: Tracks,
    meta: { title: '歌曲' }
  },
  {
    path: '/artists',
    name: 'Artists',
    component: Artists,
    meta: { title: '歌手' }
  },
  {
    path: '/artists/:artistId',
    name: 'ArtistDetail',
    component: () => import('../views/ArtistDetail.vue'),
    props: true,
    meta: { title: '歌手详情' }
  },
  {
    path: '/albums',
    name: 'Albums',
    component: Albums,
    meta: { title: '专辑' }
  },
  {
    path: '/albums/:albumId',
    name: 'AlbumDetail',
    component: () => import('../views/AlbumDetail.vue'),
    props: true,
    meta: { title: '专辑详情' }
  },
  {
    path: '/music-sources',
    name: 'MusicSourceManager',
    component: MusicSourceManager,
    meta: { title: '音乐源管理' }
  },
  {
    path: '/track/:trackId',
    name: 'TrackDetail',
    component: TrackDetail,
    props: true,
    meta: { title: '歌曲详情' }
  },
  {
    path: '/player',
    name: 'PlayerView',
    component: PlayerView,
    meta: { title: '正在播放' }
  },
  {
    path: '/settings',
    name: 'SettingsView',
    component: SettingsView,
    meta: { title: '设置' }
  },
  {
    path: '/about',
    name: 'About',
    component: About,
    meta: { title: '关于' }
  },
  {
    path: '/test',
    name: 'TestPage',
    component: TestPage,
    meta: { title: '测试页面' }
  }
]

// 创建路由实例
const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫 - 更新页面标题
router.beforeEach((to, from, next) => {
  if (to.meta && to.meta.title) {
    document.title = `${to.meta.title} - Chordial`
  } else {
    document.title = 'Chordial'
  }
  next()
})

export default router
