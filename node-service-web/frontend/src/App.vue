<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { ref, onMounted } from 'vue'

import Setup from "@/components/Setup.vue";
import { useJobsStore } from "@/stores/jobs";

const jobs = useJobsStore();

let fetchTimer: number | undefined = undefined;
let status = ref({
  connected: null,
  setup_complete: null
});

let clientConnected = ref(true);
let needsSetup = ref(false);

const statusEndpoint = "/api/status";

onMounted(async () => {
  await fetchStatus();
  needsSetup.value = !status.value.setup_complete;
  await autoRefreshStatus();
  await jobs.refreshJobs();
});

async function fetchStatus() {
  try {
    status.value = (await (await fetch(statusEndpoint)).json());
  } catch (e) {
    clientConnected.value = false;
    alert("Lost connection to client. Is client running?");
    clearTimeout(fetchTimer);
  }
}

async function autoRefreshStatus() {
  fetchTimer = window.setInterval(fetchStatus, 2000);
}

</script>

<template>
  <div class="container">
    <header class="d-flex flex-wrap align-items-center justify-content-center justify-content-md-between py-3 mb-4 border-bottom">
      <a href="/" class="d-flex align-items-center col-md-3 mb-2 mb-md-0 text-dark text-decoration-none">
        <span class="fs-5 navbar-brand">pluto</span>
      </a>
      <nav>
        <ul class="nav nav-pills">
          <li class="nav-item"><RouterLink to="/" class="nav-link">Backups</RouterLink></li>
          <li class="nav-item"><RouterLink to="/" class="nav-link">Nodes</RouterLink></li>
          <li class="nav-item"><RouterLink to="/" class="nav-link">Settings</RouterLink></li>
        </ul>
      </nav>
      <div class="col-md-3 text-end d-flex align-items-center justify-content-end">
        <div v-if="clientConnected">
          <div v-if="status.connected === true"><span class="conn-circle connected"></span><span>Connected</span></div>
          <div v-if="status.connected === false"><span class="conn-circle disconnected"></span><span>Disconnected</span></div>
          <div v-if="status.connected === null"><span>Loading...</span></div>
        </div>
        <div v-else>
          <span class="client-disconnected">Client connection lost!</span>
        </div>
      </div>
    </header>
  </div>
  <Setup v-if="needsSetup" />
  <Transition name="fade" mode="out-in">
    <RouterView v-if="status.setup_complete === true" />
  </Transition>
</template>

<style lang="scss">

@use 'assets/boostrap-icons';

@font-face {
  font-family: 'Mulish';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/fonts/Mulish-Regular.ttf') format('truetype');
}

@font-face {
  font-family: 'Mulish';
  font-style: italic;
  font-weight: 400;
  font-display: swap;
  src: url('/fonts/Mulish-Italic.ttf') format('truetype');
}

@font-face {
  font-family: 'Mulish';
  font-style: normal;
  font-weight: 600;
  font-display: swap;
  src: url('/fonts/Mulish-SemiBold.ttf') format('truetype');
}

@font-face {
  font-family: 'Mulish';
  font-style: italic;
  font-weight: 600;
  font-display: swap;
  src: url('/fonts/Mulish-SemiBoldItalic.ttf') format('truetype');
}

@font-face {
  font-family: 'Fira Mono';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url('/fonts/FiraMono-Regular.ttf') format('truetype');
}

* {
  font-family: 'Mulish', sans-serif;
}

.navbar-brand {
  font-weight: 600;
}

.conn-circle {
  height: 0.65em;
  width: 0.65em;
  border-radius: 50%;
  display: inline-block;
  margin-right: 6px;

  &.connected {
    background-color: #198754;
  }

  &.disconnected {
    background-color: #f44336;
  }
}

.client-disconnected {
  color: #f44336;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
