<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import Overview from "@/components/Overview.vue";
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
        <div> <span class="conn-circle connected"></span>Connected</div>
      </div>
    </header>
  </div>
  <RouterView />
</template>

<style lang="scss">

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
</style>
