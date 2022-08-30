import { defineStore } from 'pinia'
import { BASE_URL } from "@/constants";

export interface Node {
    pubkey: string,
    pubkey_hash: string,
    added: number,
    last_seen: number | null,
    pinned: boolean,
    label: string | null
}

const nodesEndpoint = BASE_URL + "/api/nodes";

export const useNodesStore = defineStore('nodesStore', {
    state: () => ({
        nodes: [] as Node[]
    }),
    getters: {
        getNodeCount: (state) => () => {
            return state.nodes.length;
        },
    },
    actions: {
        refreshNodes: async () => {
            try {
                useNodesStore().nodes = (await (await fetch(nodesEndpoint)).json());
            } catch (e) {
                alert("Error loading nodes: " + e);
            }
        },
    }
});
