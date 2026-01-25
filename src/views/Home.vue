<template>
  <div class="home">
    <h1>首页</h1>
    <p>欢迎使用 Chordial 应用</p>
    <div class="nav-links">
      <router-link to="/music-sources" class="nav-link">音乐源管理</router-link>
      <router-link to="/about" class="nav-link">关于我们</router-link>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Home'
}
</script>

<style scoped>
.home {
  text-align: center;
  padding: 40px 20px;
}

.nav-links {
  display: flex;
  justify-content: center;
  gap: 20px;
  margin-top: 30px;
}

.nav-link {
  padding: 10px 20px;
  background: #007bff;
  color: white;
  text-decoration: none;
  border-radius: 5px;
  transition: background-color 0.3s;
}

.nav-link:hover {
  background: #0056b3;
}
</style>