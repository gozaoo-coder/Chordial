## 问题描述
`PlayerBackground.vue` 组件中调用 AMLL 的 `BackgroundRender` 组件时，`album` 属性绑定错误地传递了 ref 对象而不是字符串值。

## 修复内容

### 文件: `src/components/player/PlayerBackground.vue`

**当前错误代码 (第 5-15 行):**
```vue
<BackgroundRender
  :key="renderKey"
  :album="stableCoverUrl"  <!-- ❌ 错误：传递的是 ref 对象 -->
  :playing="isPlaying"
  :flow-speed="flowSpeed"
  :fps="fps"
  :has-lyric="hasLyric"
  :render-scale="renderScale"
  :low-freq-volume="lowFreqVolume"
  class="amll-background"
/>
```

**修复方案 - 添加 `.value` 解包:**
```vue
<BackgroundRender
  :key="renderKey"
  :album="stableCoverUrl.value"  <!-- ✅ 正确：传递字符串值 -->
  :playing="isPlaying"
  :flow-speed="flowSpeed"
  :fps="fps"
  :has-lyric="hasLyric"
  :render-scale="renderScale"
  :low-freq-volume="lowFreqVolume"
  class="amll-background"
/>
```

## 验证步骤
1. 修复代码后重新构建项目
2. 打开播放器页面，检查背景是否正常显示
3. 切换歌曲，验证背景图片是否正确更新
4. 检查浏览器控制台是否有 AMLL 相关错误