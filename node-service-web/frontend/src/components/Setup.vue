<script setup lang="ts">
import { ref } from "vue";

let loading = ref(false);
let passphraseInput = ref("");
let passphraseGenerated = ref("");

let showPassphraseInput = ref(false);
let showResult = ref(false);

let passphraseError = ref(false);
let passphraseErrorMsg = ref("");

let setupComplete = ref(false);

const setupEndpoint = "/api/setup";

async function setup(fresh: boolean) {
  if (fresh) {
    loading.value = true;
    let response = await fetch(setupEndpoint, {
      method: "POST",
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        mnemonic: null
      }),
    });

    let json;
    if (response.ok) {
      try {
        json = await response.json();
        if (!json.success)
          throw new Error("success was false");
        passphraseGenerated.value = json.passphrase;
        showResult.value = true;
        loading.value = false;
      } catch (e) {
        alert("Something went wrong. Please try again.");
      }
    } else {
      loading.value = false;
      alert("Error: " + (await response.json()).message);
    }
  } else {
    if (!checkPassphraseLength()) {
      passphraseError.value = true;
      passphraseErrorMsg.value = "Passphrase needs to be 24 words long.";
      return;
    } else {
      passphraseError.value = false;
    }

    loading.value = true;
    let response = await fetch(setupEndpoint, {
      method: "POST",
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        mnemonic: passphraseInput.value
      }),
    });

    let json;
    if (response.ok) {
      try {
        json = await response.json();
        if (!json.success)
          throw new Error("success was false");
        loading.value = false;
        setupComplete.value = true;
      } catch (e) {
        alert("Something went wrong. Please try again.");
      }
    } else {
      loading.value = false;
      if (response.status === 400) {
        try {
          json = await response.json();
          passphraseError.value = true;
          passphraseErrorMsg.value = json.error;
        } catch (e) {
          alert("Something went wrong. Please try again.");
        }
      } else {
        alert("Error: " + (await response.json()).message);
      }
    }
  }
}

function checkPassphraseLength() {
  return passphraseInput.value.split(" ").length === 24;
}

</script>

<template>
  <Transition name="fade" appear>
    <div class="container" v-if="!setupComplete">
      <Transition name="fade">
        <div v-if="!loading" class="d-flex flex-column align-items-center">
          <h1 class="title display-3 mb-4">Welcome to pluto</h1>
          <div class="welcome border border-2 rounded-5 bg-light p-4 col-lg-7 col-md-12">
            <Transition name="fade" mode="out-in">
              <div key="1" v-if="!showPassphraseInput && !showResult">
                <h4 class="text-center">Setup</h4>
                <p>
                  Welcome to pluto! This is the first time you opened this application,
                  please choose if you want to do a a fresh backup or restore an existing one.
                  Pluto lets you easily backup your data to other people securely for free,
                  all you need is some extra disk space, so they can make a backup of their own!
                </p>
                <p>
                  Creating a new backup will generate a 24-word passphrase that you can use to restore your data.
                  You can also restore an existing backup if you already have a passphrase.
                </p>
                <div class="buttons d-flex mt-4 flex-column align-items-center">
                  <button type="button" class="btn btn-dark btn-lg mb-2" @click="setup(true)">New backup</button>
                  <button type="button" class="btn btn-outline-dark btn-lg" @click="showPassphraseInput = true">
                    Restore
                  </button>
                </div>
              </div>
              <div key="2" v-else-if="showPassphraseInput && !showResult">
                <h4 class="text-center">Passphrase recovery</h4>
                <p>
                  Please enter your previously saved 24-word passphrase to restore existing backups.
                </p>
                <div class="form-floating">
                  <textarea :class="{ 'is-invalid': passphraseError }" class="form-control"
                            placeholder="Enter your passphrase" id="passphrase"
                            v-model="passphraseInput"></textarea>
                  <label for="passphrase">Passphrase</label>
                  <div class="invalid-feedback">
                    {{ passphraseErrorMsg }}
                  </div>
                </div>
                <div class="d-flex justify-content-end">
                  <button type="button" class="btn btn-outline-dark btn-lg mt-3 me-2"
                          @click="showPassphraseInput = false">Back
                  </button>
                  <button type="button" class="btn btn-dark btn-lg mt-3" @click="setup(false)">Continue</button>
                </div>
              </div>
              <div key="3" v-else-if="showResult">
                <h4 class="text-center">Setup successful!</h4>
                <p>
                  Please take good note of your new 24-word passphrase.
                  It serves as a way to restore your backup, should you ever need it.
                  You can write it down, print it, or save it digitally. Store it in a safe place, ideally in multiple
                  copies.
                  Just keep in mind that anyone who gets access to this passphrase will be able to restore your backup!
                </p>
                <div class="passphrase-generated p-1 text-center">
                  {{ passphraseGenerated }}
                </div>
                <div class="d-flex justify-content-end">
                  <button type="button" class="btn btn-dark btn-lg mt-3" @click="setupComplete = true">Done</button>
                </div>
              </div>
            </Transition>
          </div>
        </div>
        <div v-else class="d-flex flex-column align-items-center">
          <div class="spinner-border" role="status">
            <span class="visually-hidden">Loading...</span>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped lang="scss">
.title {
  font-weight: 600;
}

.buttons {
  button {
    width: 250px;
  }
}

.passphrase-generated {
  font-family: 'Fira Mono', monospace;
  font-size: 1.4rem;
}

#passphrase {
  font-family: 'Fira Mono', monospace;
  font-size: 1.2em;
  min-height: 80px;
  height: 160px;
  max-height: 260px;
}

.welcome {
  border-radius: .4em;
}
</style>
