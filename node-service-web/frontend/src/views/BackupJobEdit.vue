<template>
  <div class="container">
    <h3 class="text-center">Edit backup job</h3>
    <template v-if="job">
      <BackupJob :job="job" :new="false" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { useJobsStore } from '@/stores/jobs'
import type { Job } from '@/stores/jobs'
import BackupJob from "@/components/BackupJob.vue";

import { onMounted, ref, watch } from "vue";
import type { Ref } from "vue";

const jobs = useJobsStore();
let job: Ref<Job|undefined> = ref(undefined);

const props = defineProps<{
  id: number
}>();

onMounted(() => {
  job.value = jobs.getJobById(props.id);
})

watch(jobs, (jobsNew, jobsOld) => {
  job.value = jobs.getJobById(props.id);
});

</script>

<style scoped>

</style>
