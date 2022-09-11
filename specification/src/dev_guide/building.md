# Building and running

## Building
To build the entire project, you need Rust with Cargo (the project targets the latest stable version).
The frontend for node binary requires Node.js (target version is 16.x) and npm.
Additionally, coordinator requires libpq for Postgres.

To install Rust and Cargo, simply follow the official instructions: <https://www.rust-lang.org/tools/install>.
Node.js offers installation instructions on this page: <https://nodejs.org/en/download/>.

Additional dependencies (libpq) can be installed on Fedora-like distros with this command:

```bash
sudo dnf install libpq
```

For Debian-like distros use this command:

```bash
sudo apt install libpq5
```

Now with all system dependencies installed, run the following commands to build:

```bash
cd crates/node-service-web/frontend  # go to frontend folder
npm install                          # install dependencies for frontend
npm run build                        # build frontend
cd ../../..                          # go back to root folder
cargo build --release                # install Rust dependencies and build all components
```

## Running

### Node
The node binary is standalone, it has all of the necessary files (libraries, frontend assets etc.) embedded.
You can run it using Cargo with this command:

```bash
cargo run --bin pluto-node-service-web --release
```

Config (for coordinator host and keys) is located in `crates/node/src/config.rs`.

### Coordinator
Coordinator relies on a Postgres database server and a MQTT server (we're using Mosquitto).
The easiest way to run everything is using the provided development Docker Compose config.
It will automatically build a Docker container that contains the coordinator binary and necessary dependencies.
We're using a multi-step build process with cargo-chef, so dependencies don't have to be rebuilt each time,
speeding up the process significantly.

Config for coordinator is stored in a `.env` file, the provided `.env.example` file will work
with Docker Compose and default configuration.

Use default configuration:
```bash
cp .env.example .env
```

Install Docker if you haven't already, and run coordinator using this command:

```bash
docker-compose up --build
```

MQTT server will be exposed on `localhost:1883`,
which is the default host that is used by node, so connecting should just work.

If you run into permission issues on a system with SELinux, try disabling it.

To generate a new keypair, run the coordinator binary with `keygen` as the first argument,
it will generate new random keys and print them out. Node config needs to be changed accordingly.
