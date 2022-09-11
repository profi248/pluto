<script setup lang="ts">
import type { Node } from '@/stores/nodes'
import { useNodesStore } from '@/stores/nodes';
import { onMounted } from "vue";

// @ts-ignore
import emojify from 'emojify-hashes'

const nodes = useNodesStore();

onMounted(async () => {
  await nodes.refreshNodes();
});
</script>

<template>
  <div class="container">
    <h2 class="text-center mb-3">Nodes</h2>
    <div class="d-flex justify-content-center">
      <div class="col-md-8 col-lg-6 col-12">
        <div class="card" v-if="nodes.getNodeCount() > 0">
          <ul class="list-group list-group-flush">
            <li class="list-group-item d-flex align-items-center" v-for="node in nodes.nodes">
              <span class="pubkey">{{ emojify(node.pubkey_hash).join(" ") }}</span>
              <div class="ms-auto">
                <button type="button" class="btn btn-outline-dark me-2"><i class="bi bi-trash3"></i></button>
              </div>
            </li>
          </ul>
        </div>
        <div class="card bg-light" v-else>
          <div class="card-body">
            <p class="card-text text-center">
              <em>All nodes that we communicated with will be shown here. You can also add one manually.</em>
            </p>
          </div>
        </div>
        <div class="d-flex justify-content-end mt-2">
          <router-link to="/nodes">
            <button type="button" class="btn btn-dark"><i class="bi bi-plus-circle"></i> Add node manually</button>
          </router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pubkey {
  font-size: 250%;
}
</style>
