## 问题分析

### 根本原因
PlayerBackground.vue 第 7 行代码：
```vue
:album="stableCoverUrl.value"
```

在 Vue 模板中，ref 会被自动解包，不需要手动访问 `.value`。当前代码传递的是 ref 对象本身，而不是字符串 URL，导致 AMLL 的 BackgroundRender 组件无法正确加载专辑封面。

### 症状
1. 播放音乐时点击底栏看歌词，背景不显示
2. 暂停音乐时背景突然显示
3. 点击播放时程序卡死
4. 控制台报错 `EXT_color_buffer_float not supported`（这是 WebGL 警告，不是根本原因）

### 修复方案
将第 7 行从：
```vue
:album="stableCoverUrl.value"
```
改为：
```vue
:album="stableCoverUrl"
```

### 验证步骤
1. 修复代码后重新构建项目
2. 打开播放器页面，播放音乐
3. 点击底栏查看歌词，验证背景是否正常显示
4. 切换播放/暂停状态，验证不会卡死
5. 切换歌曲，验证背景图片是否正确更新