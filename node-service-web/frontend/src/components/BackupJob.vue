<template>
  <div class="d-flex justify-content-center mt-4">
    <div class="col-md-6 col-lg-4 col-12">
      <div class="form-floating">
        <input type="text" class="form-control" v-model.trim="jobName" id="job-name" placeholder="Job name">
        <label for="job-name">Job name</label>
      </div>
      <div class="d-flex justify-content-center">
        <button type="button" class="btn btn-outline-dark btn-lg mt-3" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useJobsStore } from '@/stores/jobs'
import type { Job } from "@/stores/jobs";
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";

let jobName = ref('');

let jobs = useJobsStore();

const router = useRouter();

const props = defineProps<{
  job?: Job,
  new: boolean
}>()


onMounted(async () => {
  if (!props.new && !props.job) throw new Error("Job prop is not set");
  if (!props.new) jobName.value = props.job?.name!;
});

async function save() {
  if (jobName.value.length === 0) {
    alert("Job name is required");
    return;
  }

  if (jobName.value.length > 255) {
    alert("Job name is too long");
    return;
  }

  try {
    let job: any = {};
    if (props.new) {
      job.name = jobName.value;

      await jobs.createJob(job);
    } else {
      job = props.job;
      job.name = jobName.value;

      await jobs.updateJob(job);
    }
  } catch (e) {
    alert("Error: " + e);
  }

  await router.push('/');
}

</script>

<style scoped>

</style>
