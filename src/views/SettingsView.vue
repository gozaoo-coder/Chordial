<template>
  <div class="settings-shell">
    <aside class="settings-nav">
      <h1 class="nav-title">设置</h1>
      <router-link :to="{ name: 'SettingsGeneral' }" class="nav-item" active-class="active">
        <i class="bi bi-sliders"></i>
        <span>通用</span>
      </router-link>
      <router-link :to="{ name: 'SettingsP2p' }" class="nav-item" active-class="active">
        <i class="bi bi-broadcast"></i>
        <span>P2P 资源共享</span>
      </router-link>
      <router-link :to="{ name: 'SettingsIndex' }" class="nav-item" active-class="active">
        <i class="bi bi-house"></i>
        <span>设置首页</span>
      </router-link>
    </aside>
    <section class="settings-main">
      <router-view />
    </section>
  </div>
</template>

<script setup>
</script>

<style scoped>
.settings-shell {
  display: flex;
  height: 100%;
  min-height: calc(100vh - var(--header-height));
}

.settings-nav {
  width: 240px;
  flex-shrink: 0;
  padding: 24px 16px;
  border-right: 1px solid var(--border-light, #e8e8e8);
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: var(--bg-glass, transparent);
  backdrop-filter: saturate(180%) blur(20px);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
}

.nav-title {
  font-size: 24px;
  font-weight: 800;
  color: var(--text-primary, #333);
  margin: 0 8px 24px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  color: var(--text-secondary, #666);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  transition: all 0.2s ease;
}

.nav-item:hover {
  background: var(--bg-hover, rgba(0, 0, 0, 0.04));
  color: var(--text-primary, #333);
}

.nav-item.active {
  background: var(--primary-light, rgba(0, 120, 215, 0.1));
  color: var(--primary-color, #0078d7);
}

.nav-item i {
  font-size: 16px;
}

.settings-main {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

@media (max-width: 768px) {
  .settings-shell {
    flex-direction: column;
  }

  .settings-nav {
    width: 100%;
    flex-direction: row;
    overflow-x: auto;
    padding: 12px;
    border-right: none;
    border-bottom: 1px solid var(--border-light, #e8e8e8);
  }

  .nav-title {
    display: none;
  }

  .nav-item {
    flex-shrink: 0;
    padding: 8px 14px;
  }

  .settings-main {
    padding: 16px;
  }
}
</style>
