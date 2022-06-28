<script setup lang="ts">
import { useJobsStore } from '@/stores/jobs';
import type { Job } from '@/stores/jobs'

const jobs = useJobsStore();

async function deleteJob(job: Job) {
  if (confirm(`Are you sure you want to delete job "${job.name}"?`)) {
    try {
      await jobs.deleteJob(job.job_id);
    } catch (e) {
      alert("Error: " + e);
    }
  }
}
</script>

<template>
  <main>
    <div class="container">
      <h2 class="text-center mb-3">Backup jobs</h2>
      <div class="d-flex justify-content-center">
        <div class="col-md-8 col-lg-6 col-12">
          <div class="card" v-if="jobs.getJobCount() > 0">
            <ul class="list-group list-group-flush">
              <li class="list-group-item d-flex align-items-center" v-for="backupJob in jobs.jobs">
                {{ backupJob.name }}
                <div class="ms-auto">
                  <button type="button" class="btn btn-outline-dark me-2" @click="deleteJob(backupJob)"><i class="bi bi-trash3"></i></button>
                  <router-link :to="'/backup_jobs/' + backupJob.job_id">
                    <button type="button" class="btn btn-outline-dark"><i class="bi bi-pencil-square"></i></button>
                  </router-link>
                </div>
              </li>
            </ul>
          </div>
          <div class="card bg-light" v-else>
            <div class="card-body">
              <p class="card-text text-center">
                <em>No backup jobs exist yet, create a new one!</em>
              </p>
            </div>
          </div>
          <div class="d-flex justify-content-end mt-2">
            <router-link to="/backup_jobs/new">
              <button type="button" class="btn btn-dark"><i class="bi bi-plus-circle"></i> Add new</button>
            </router-link>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>
