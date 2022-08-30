import { defineStore } from 'pinia'
import { BASE_URL } from "@/constants";

export interface Path {
    path_id: number | null,
    path: string,
    path_type: "Folder" | "IgnorePattern"
}

export interface JobCreate {
    name: string,
    paths: Path[]
}

export interface JobUpdate {
    job_id: number,
    name: string,
    paths: Path[]
    last_ran: number | null
}

export interface Job {
    job: {
        job_id: number | null,
        name: string,
        created: number | null,
        last_ran: number | null,
    },
    paths: Path[]
}

const backupJobEndpoint = BASE_URL + "/api/backup_jobs";
const jobPathEndpointName = "paths";

export const useJobsStore = defineStore('jobsStore', {
    state: () => ({
        jobs: [] as Job[]
    }),
    getters: {
        getJobById: (state) => (id: number) => {
            return state.jobs.find((job) => job.job.job_id === id)
        },
        getJobCount: (state) => () => {
            return state.jobs.length;
        },
        getJobFolders: (state) => (job_id: number) => {
            let job = useJobsStore().getJobById(job_id);
            return job?.paths.filter((path) => path.path_type === "Folder");
        },
        getJobIgnorePatterns: (state) => (job_id: number) => {
            let job = useJobsStore().getJobById(job_id);
            return job?.paths.filter((path) => path.path_type === "IgnorePattern");
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
        createJob: async (job: JobCreate) => {
            // is browser support good enough?
            let jobJson = structuredClone(job);
            delete jobJson.paths;

            console.log(job);
            let response = await fetch(backupJobEndpoint, {
                method: "POST",
                headers: {
                    'Accept': 'application/json',
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(jobJson),
            });

            let json;
            if (response.ok) {
                json = await response.json();
                if (!json.success) throw new Error("Success was false");
            } else {
                throw new Error((await response.json()).error);
            }

            // create paths for job
            let jobId = json.job_id;
            for (const path of job.paths) {
                let pathJson = {
                    path: path.path,
                    path_type: path.path_type
                };

                let response: Response = await fetch(`${backupJobEndpoint}/${jobId}/${jobPathEndpointName}`,
                    {
                        method: "POST",
                        headers: {
                            'Accept': 'application/json',
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify(pathJson),
                    });

                if (response.ok) {
                    json = await response.json();
                    if (!json.success) throw new Error("Success was false");
                } else {
                    throw new Error((await response.json()).error);
                }
            }

            await useJobsStore().refreshJobs();
        },
        updateJob: async (job: JobUpdate) => {
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

            // crete/update paths for job
            let jobId = job.job_id;

            for (const path of job.paths) {
                let pathJson: Path = {
                    path_id: null,
                    path: path.path,
                    path_type: path.path_type
                };

                let response;
                if (path.path_id === null) {
                    response = await fetch(`${backupJobEndpoint}/${jobId}/${jobPathEndpointName}`,
                        {
                            method: "POST",
                            headers: {
                                'Accept': 'application/json',
                                'Content-Type': 'application/json'
                            },
                            body: JSON.stringify(pathJson),
                        });
                } else {
                    pathJson.path_id = path.path_id;
                    response = await fetch(`${backupJobEndpoint}/${jobId}/${jobPathEndpointName}/${path.path_id}`,
                        {
                            method: "PUT",
                            headers: {
                                'Accept': 'application/json',
                                'Content-Type': 'application/json'
                            },
                            body: JSON.stringify(pathJson),
                        });
                }

                if (response.ok) {
                    json = await response.json();
                    if (!json.success) throw new Error("Success was false");
                } else {
                    throw new Error((await response.json()).error);
                }
            }

            // delete paths for job
            let toDelete: number[] = [];
            useJobsStore().getJobById(jobId)?.paths.forEach((oldPath) => {
                if (!job.paths.find((newPath) => newPath.path_id === oldPath.path_id)) {
                    toDelete.push(oldPath.path_id!);
                }
            });

            for (const pathId of toDelete) {
                let response: Response = await fetch(`${backupJobEndpoint}/${jobId}/${jobPathEndpointName}/${pathId}`,
                    {
                        method: "DELETE",
                        headers: {
                            'Accept': 'application/json',
                            'Content-Type': 'application/json'
                        },
                    });

                if (response.ok) {
                    json = await response.json();
                    if (!json.success) throw new Error("Success was false");
                } else {
                    throw new Error((await response.json()).error);
                }
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
