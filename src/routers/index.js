import { createRouter, createWebHistory } from 'vue-router'
import { perf } from '@/utils/performanceMonitor.js'

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
const GeneralSettingsView = () => import('../views/settings/GeneralSettingsView.vue')
const P2pSettingsView = () => import('../views/settings/P2pSettingsView.vue')
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
    name: 'Settings',
    component: SettingsView,
    meta: { title: '设置' },
    children: [
      {
        path: '',
        name: 'SettingsIndex',
        component: () => import('../views/settings/SettingsIndex.vue'),
        meta: { title: '设置' }
      },
      {
        path: 'general',
        name: 'SettingsGeneral',
        component: GeneralSettingsView,
        meta: { title: '通用设置' }
      },
      {
        path: 'p2p',
        name: 'SettingsP2p',
        component: P2pSettingsView,
        meta: { title: 'P2P 资源共享' }
      }
    ]
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

// 路由守卫 - 更新页面标题 + 性能监控
let _routeTimerLabel = null

router.beforeEach((to, from, next) => {
  if (to.meta && to.meta.title) {
    document.title = `${to.meta.title} - Chordial`
  } else {
    document.title = 'Chordial'
  }

  // 页面导航性能计时 — 捕获 start() 返回值，避免同名路由重复计时
  if (from.name) {
    _routeTimerLabel = perf.start(`route:${from.name}→${to.name}`)
  } else {
    _routeTimerLabel = perf.start(`route:→${to.name}`)
  }
  next()
})

router.afterEach(() => {
  if (_routeTimerLabel) {
    perf.end(_routeTimerLabel)
    _routeTimerLabel = null
  }
})

export default router
