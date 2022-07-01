<template>
  <div class="d-flex justify-content-center mt-4">
    <div class="col-md-6 col-lg-4 col-12">
      <div class="form-floating">
        <input type="text" class="form-control" v-model.trim="jobName" id="job-name" placeholder="Job name">
        <label for="job-name">Job name</label>
      </div>
      <h5 class="text-center mt-3">Folders</h5>
      <div class="card mt-3">
        <ul class="list-group list-group-flush">
          <li v-for="(folder, index) in folders" class="list-group-item d-flex p-4">
            <div class="flex-grow-1">
              <input type="text" v-model="folder.path" class="form-control" :id="`path-${index}`" placeholder="/home/user/">
            </div>
            <button type="button" class="btn btn-outline-dark ms-2 height-fit-content" @click="folders.splice(index, 1)">
              <i class="bi bi-trash3"></i>
            </button>
          </li>
        </ul>
      </div>
      <div class="d-flex justify-content-end mt-2">
        <button type="button" class="btn btn-dark" @click="folders.push( { path_id: null, path: '', path_type: 'Folder' } )">
          <i class="bi bi-folder-plus"></i> Add folder
        </button>
      </div>
      <h5 class="text-center mt-3">Ignore patterns</h5>
      <div class="card mt-3">
        <ul v-if="ignores.length > 0" class="list-group list-group-flush">
          <li v-for="(ignore, index) in ignores" class="list-group-item d-flex p-4">
            <div class="flex-grow-1">
              <input type="text" v-model="ignore.path" class="form-control" :id="`path-${index}`" placeholder="*.log">
            </div>
            <button type="button" class="btn btn-outline-dark ms-2 height-fit-content" @click="ignores.splice(index, 1)">
              <i class="bi bi-trash3"></i>
            </button>
          </li>
        </ul>
        <div v-else class="card-body">
          <p class="card-text text-center">
            <em>Ignore patterns let you exclude files and folders from backup with a wildcard expression.</em>
          </p>
        </div>
      </div>
      <div class="d-flex justify-content-end mt-2">
        <button type="button" class="btn btn-dark" @click="ignores.push( { path_id: null, path: '', path_type: 'IgnorePattern' } )">
          <i class="bi bi-plus-circle-dotted"></i> Add ignore pattern
        </button>
      </div>
      <div class="d-flex justify-content-center">
        <button type="button" class="btn btn-outline-dark btn-lg mt-5" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useJobsStore } from '@/stores/jobs'
import type { Job, Path, JobCreate, JobUpdate } from '@/stores/jobs'
import { onMounted, ref, watch } from "vue";
import type { Ref } from "vue";
import { useRouter } from "vue-router";

let jobName = ref('');
let folders: Ref<Path[]> = ref([]);
let ignores: Ref<Path[]> = ref([]);

let jobs = useJobsStore();

const router = useRouter();

const props = defineProps<{
  job_id?: number,
  new: boolean
}>()

onMounted(async () => {
  if (!props.new && !props.job_id) throw new Error("Job prop is not set");
  if (!props.new) {
    jobName.value = jobs.getJobById(props.job_id!)?.job.name!;
    folders.value = jobs.getJobFolders(props.job_id!)!;
    ignores.value = jobs.getJobIgnorePatterns(props.job_id!)!;
  } else {
    folders.value = [{
        path_id: null,
        path: '',
        path_type: 'Folder'
      }];
  }
});

watch(jobs, async (jobsNew, jobsOld) => {
  if (!props.new) {
    jobName.value = jobs.getJobById(props.job_id!)?.job.name!;

    // don't override paths if user has already changed them
    if (folders.value.length === 1 && folders.value[0].path === '')
      folders.value = jobs.getJobFolders(props.job_id!)!;

    if (ignores.value.length === 0)
      ignores.value = jobs.getJobIgnorePatterns(props.job_id!)!;
  }
});

watch(folders, async (foldersNew, foldersOld) => {
  if (foldersNew.length === 0) folders.value = [{
      path_id: null,
      path: '',
      path_type: 'Folder'
    }];
}, { deep: true });

function mergePaths(folders: Path[], ignores: Path[]) {
  let paths: Path[] = [];

  folders.forEach(folder => {
    paths.push({
      path_id: folder.path_id,
      path: folder.path,
      path_type: 'Folder'
    });
  });

  ignores.forEach(ignore => {
    paths.push({
      path_id: ignore.path_id,
      path: ignore.path,
      path_type: 'IgnorePattern'
    });
  });

  return paths;
}

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
    let job: JobCreate | JobUpdate;
    if (props.new) {
      job = {
        name: jobName.value,
        paths: mergePaths(folders.value, ignores.value)
      };
      job.name = jobName.value;
      job.paths = mergePaths(folders.value, ignores.value);

      await jobs.createJob(job);
    } else {
      let jobOld: Job = jobs.getJobById(props.job_id!)!;
      let jobNew: JobUpdate = {
        job_id: props.job_id!,
        name: jobName.value,
        last_ran: jobOld.job.last_ran!,
        paths: mergePaths(folders.value, ignores.value)
      };

      await jobs.updateJob(jobNew);
    }
  } catch (e) {
    alert("Error: " + e);
  }

  await router.push('/');
}

</script>

<style scoped>
.height-fit-content {
  height: fit-content;
}
</style>
