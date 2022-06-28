<template>
  <div class="container">
    <h3 class="text-center">New backup job</h3>
    <BackupJob :new="true"/>
  </div>
</template>

<script setup lang="ts">
import BackupJob from "@/components/BackupJob.vue";

</script>

<style scoped>

</style>
