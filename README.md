# Chordial

一个优雅、强大的桌面音乐软件，使用现代Web技术构建，提供流畅的音乐体验。

## ✨ 功能特点

- 📁 **音乐库管理** - 自动扫描和管理多端音乐文件
- 🎨 **现代UI设计** - 简洁、美观的用户界面

## 🛠️ 技术栈

- **前端框架**: Vue 3
- **构建工具**: Vite
- **桌面应用**: Tauri
- **开发语言**: JavaScript / Rust

## 📦 安装

### 前提条件

- Node.js (>= 18.x)
- Rust (>= 1.75)
- Tauri CLI

### 安装步骤

1. 克隆仓库
```bash
git clone https://github.com/yourusername/chordial.git
cd chordial
```

2. 安装依赖
```bash
npm install
```

## 🚀 运行

### 开发模式

```bash
npm run tauri dev
```

这将启动开发服务器并打开应用程序窗口。

## 🔧 开发

### 项目结构

```
chordial/
├── src/                 # 前端代码
│   ├── assets/          # 静态资源
│   ├── components/      # Vue组件
│   ├── App.vue          # 主应用组件
│   └── main.js          # 入口文件
├── src-tauri/           # Tauri后端代码
│   ├── src/             # Rust源代码
│   ├── icons/           # 应用图标
│   ├── Cargo.toml       # Rust依赖配置
│   └── tauri.conf.json  # Tauri配置
├── package.json         # 前端依赖配置
├── vite.config.js       # Vite配置
└── README.md            # 项目说明文档
```

### 主要脚本

- `npm run dev` - 启动Vite开发服务器
- `npm run build` - 构建前端代码
- `npm run tauri dev` - 启动Tauri开发模式
- `npm run tauri build` - 构建生产版本

## 📦 构建

### 构建生产版本

```bash
npm run tauri build
```

构建完成后，可执行文件将位于 `src-tauri/target/release/` 目录中。

## 🎨 自定义主题

Chordial支持自定义主题，您可以在设置中切换深色/浅色主题，或者根据自己的喜好调整颜色方案。

## 🤝 贡献

欢迎贡献代码、提出问题或建议！

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 联系方式

- 项目链接: [https://github.com/yourusername/chordial](https://github.com/yourusername/chordial)
- 问题反馈: [https://github.com/yourusername/chordial/issues](https://github.com/yourusername/chordial/issues)

## 📝 更新日志

### v0.1.0 (2025-01-24)
- 初始版本
- 基本音乐播放功能
- 和弦识别功能
- 钢琴可视化

---

**Chordial** - 让音乐更亲近 🎶