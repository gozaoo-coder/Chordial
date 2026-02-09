<script setup>
const appVersion = '0.1.0';
</script>

<template>
  <div class="about-page">
    <div class="about-card card">
      <div class="app-logo">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <circle cx="12" cy="12" r="3"/>
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3"/>
        </svg>
      </div>
      <h1 class="app-name">Chordial</h1>
      <p class="app-version">版本 {{ appVersion }}</p>
      <p class="app-description">
        Chordial 是一款现代化的本地音乐播放器，支持多种音频格式，
        提供优雅的用户界面和流畅的音乐体验。
      </p>
      
      <div class="features">
        <h3 class="features-title">主要功能</h3>
        <ul class="features-list">
          <li>支持 MP3、FLAC、WAV、M4A 等多种音频格式</li>
          <li>自动读取音乐元数据和专辑封面</li>
          <li>智能音乐库管理</li>
          <li>歌词显示支持</li>
          <li>深色模式</li>
        </ul>
      </div>

      <div class="tech-stack">
        <h3 class="tech-title">技术栈</h3>
        <div class="tech-tags">
          <span class="tech-tag">Vue 3</span>
          <span class="tech-tag">Tauri</span>
          <span class="tech-tag">Rust</span>
          <span class="tech-tag">Vite</span>
        </div>
      </div>

      <div class="copyright">
        <p>© 2024 Chordial. All rights reserved.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-page {
  max-width: 600px;
  margin: 0 auto;
  padding: 40px 20px;
}

.about-card {
  text-align: center;
  padding: 40px;
}

.app-logo {
  width: 80px;
  height: 80px;
  margin: 0 auto 24px;
  background: linear-gradient(135deg, var(--primary-color, #0078d7), #005a9e);
  border-radius: var(--radius-lg, 18px);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.app-logo svg {
  width: 40px;
  height: 40px;
}

.app-name {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary, #333);
  margin: 0 0 8px 0;
}

.app-version {
  font-size: 14px;
  color: var(--text-secondary, #666);
  margin: 0 0 24px 0;
}

.app-description {
  font-size: 14px;
  color: var(--text-secondary, #666);
  line-height: 1.6;
  margin: 0 0 32px 0;
}

.features {
  text-align: left;
  margin-bottom: 32px;
}

.features-title,
.tech-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 16px 0;
}

.features-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.features-list li {
  font-size: 14px;
  color: var(--text-secondary, #666);
  padding: 8px 0;
  padding-left: 24px;
  position: relative;
}

.features-list li::before {
  content: '✓';
  position: absolute;
  left: 0;
  color: var(--primary-color, #0078d7);
  font-weight: 600;
}

.tech-stack {
  text-align: left;
  margin-bottom: 32px;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tech-tag {
  padding: 6px 12px;
  background: var(--bg-tertiary, #e8e8ed);
  border-radius: 20px;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.copyright {
  padding-top: 24px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
}

.copyright p {
  font-size: 12px;
  color: var(--text-tertiary, #999);
  margin: 0;
}

@media (max-width: 767px) {
  .about-page {
    padding: 20px 16px;
  }
  
  .about-card {
    padding: 24px;
  }
  
  .app-name {
    font-size: 24px;
  }
}

@media (prefers-color-scheme: dark) {
  .app-name,
  .features-title,
  .tech-title {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
