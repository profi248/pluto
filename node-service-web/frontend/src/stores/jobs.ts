import { defineStore } from 'pinia'

export interface Job {
  job_id: number,
  name: string,
  created: number,
  last_ran: number | null,
}

const backupJobEndpoint = "/api/backup_jobs";

export const useJobsStore = defineStore('jobsStore', {
  state: () => ({
    jobs: [] as Job[]
  }),
  getters: {
    getJobById: (state) => (id: number) => {
        return state.jobs.find((job) => job.job_id === id)
    },
    getJobCount: (state) => () => {
        return state.jobs.length;
    }
  },
  actions: {
    refreshJobs: async () => {
      // why can't we use `this` here??
      try {
        useJobsStore().jobs = (await (await fetch(backupJobEndpoint)).json()).jobs;
      } catch (e) {
        alert("Error loading backup jobs: " + e);
      }
    },
    createJob: async (job: Job) => {
      let response = await fetch(backupJobEndpoint, {
        method: "POST",
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(job),
      });

      let json;
      if (response.ok) {
        json = await response.json();
        if (!json.success) throw new Error("Success was false");
      } else {
        throw new Error((await response.json()).error);
      }

      await useJobsStore().refreshJobs();
    },
    updateJob: async (job: Job) => {
      let response = await fetch(backupJobEndpoint + `/${job.job_id}`, {
        method: "PUT",
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(job),
      });

      let json;
      if (response.ok) {
        json = await response.json();
        if (!json.success) throw new Error("Success was false");
      } else {
        throw new Error((await response.json()).error);
      }

      await useJobsStore().refreshJobs();
    },
    deleteJob: async (id: number) => {
        let response = await fetch(backupJobEndpoint + `/${id}`, {
          method: "DELETE",
          headers: {
            'Accept': 'application/json'
          },
        });

        if (response.ok) {
          let json = await response.json();
          if (!json.success) throw new Error("Success was false");
        } else {
          throw new Error("Error deleting job: " + (await response.json()).error);
        }

        await useJobsStore().refreshJobs();
    }
  },
})
