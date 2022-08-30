import { createRouter, createWebHistory } from 'vue-router'

import BackupJobs from '../views/BackupJobs.vue'
import BackupJobEdit from '../views/BackupJobEdit.vue'
import BackupJobNew from '../views/BackupJobNew.vue'

import Nodes from '../views/Nodes.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/backup_jobs'
    },
    {
      path: '/backup_jobs',
      name: 'backup-jobs',
      component: BackupJobs
    },
    {
      path: '/backup_jobs/:id',
      name: 'backup-job-edit',
      component: BackupJobEdit,
      props: (route) => {
        return { id: Number(route.params.id) }
      }
    },
    {
      path: '/backup_jobs/new',
      name: 'backup-job-new',
      component: BackupJobNew,
    },
    {
        path: '/nodes',
        name: 'nodes',
        component: Nodes
    }
  ]
})

export default router
