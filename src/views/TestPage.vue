<script setup>
import { ref } from 'vue';

const testResults = ref([]);

const runTests = () => {
  testResults.value = [];
  
  // Test 1: Check if API modules are available
  try {
    const modules = import.meta.glob('../api/**/*.js');
    addResult('API Modules', '通过', `找到 ${Object.keys(modules).length} 个 API 模块`);
  } catch (error) {
    addResult('API Modules', '失败', error.message);
  }
  
  // Test 2: Check if class modules are available
  try {
    const modules = import.meta.glob('../class/**/*.js');
    addResult('Class Modules', '通过', `找到 ${Object.keys(modules).length} 个类模块`);
  } catch (error) {
    addResult('Class Modules', '失败', error.message);
  }
  
  // Test 3: Check if components are available
  try {
    const components = import.meta.glob('../components/**/*.vue');
    addResult('Components', '通过', `找到 ${Object.keys(components).length} 个组件`);
  } catch (error) {
    addResult('Components', '失败', error.message);
  }
  
  // Test 4: Check if views are available
  try {
    const views = import.meta.glob('./*.vue');
    addResult('Views', '通过', `找到 ${Object.keys(views).length} 个视图`);
  } catch (error) {
    addResult('Views', '失败', error.message);
  }
  
  // Test 5: Check localStorage
  try {
    localStorage.setItem('test', 'test');
    localStorage.removeItem('test');
    addResult('LocalStorage', '通过', '可以正常读写');
  } catch (error) {
    addResult('LocalStorage', '失败', error.message);
  }
};

const addResult = (name, status, message) => {
  testResults.value.push({
    name,
    status,
    message,
    time: new Date().toLocaleTimeString()
  });
};

const clearResults = () => {
  testResults.value = [];
};
</script>

<template>
  <div class="test-page">
    <div class="page-header">
      <h1 class="page-title">测试页面</h1>
      <p class="page-subtitle">运行系统测试</p>
    </div>

    <div class="test-actions">
      <button class="btn btn-primary" @click="runTests">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="5 3 19 12 5 21 5 3"/>
        </svg>
        运行测试
      </button>
      <button class="btn btn-secondary" @click="clearResults">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
        </svg>
        清除结果
      </button>
    </div>

    <div v-if="testResults.length > 0" class="test-results card">
      <h3 class="results-title">测试结果</h3>
      <div class="results-list">
        <div 
          v-for="(result, index) in testResults" 
          :key="index"
          class="result-item"
          :class="{ 'result-pass': result.status === '通过', 'result-fail': result.status === '失败' }"
        >
          <div class="result-header">
            <span class="result-name">{{ result.name }}</span>
            <span class="result-status">{{ result.status }}</span>
          </div>
          <p class="result-message">{{ result.message }}</p>
          <span class="result-time">{{ result.time }}</span>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/>
      </svg>
      <h3 class="empty-state-title">准备就绪</h3>
      <p class="empty-state-desc">点击"运行测试"按钮开始测试</p>
    </div>
  </div>
</template>

<style scoped>
.test-page {
  max-width: 800px;
  margin: 0 auto;
}

.test-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.test-actions .btn {
  display: flex;
  align-items: center;
  gap: 8px;
}

.test-actions .btn svg {
  width: 18px;
  height: 18px;
}

.test-results {
  padding: 24px;
}

.results-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #333);
  margin: 0 0 20px 0;
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-item {
  padding: 16px;
  border-radius: var(--radius-md, 8px);
  background: var(--bg-tertiary, #e8e8ed);
}

.result-item.result-pass {
  background: rgba(34, 197, 94, 0.1);
  border-left: 4px solid #22c55e;
}

.result-item.result-fail {
  background: rgba(239, 68, 68, 0.1);
  border-left: 4px solid #ef4444;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.result-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.result-status {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 8px;
  border-radius: 4px;
  background: var(--bg-secondary, #ffffff);
}

.result-pass .result-status {
  color: #22c55e;
}

.result-fail .result-status {
  color: #ef4444;
}

.result-message {
  font-size: 13px;
  color: var(--text-secondary, #666);
  margin: 0 0 8px 0;
}

.result-time {
  font-size: 11px;
  color: var(--text-tertiary, #999);
}

@media (max-width: 767px) {
  .test-actions {
    flex-direction: column;
  }
  
  .test-actions .btn {
    justify-content: center;
  }
}

@media (prefers-color-scheme: dark) {
  .results-title,
  .result-name {
    color: var(--text-primary, #f6f6f6);
  }
}
</style>
