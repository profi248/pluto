<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import Overview from "@/components/Overview.vue";
</script>

<template>
  <header>
    <Overview msg="hello world" />
      <nav>
        <RouterLink to="/">Home</RouterLink>
        <RouterLink to="/about">About</RouterLink>
      </nav>
  </header>
  <RouterView />
</template>

<style>

</style>
