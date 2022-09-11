# Source Structure

```
mosqitto_configs/
specification/
crates/
    backup/
    cli/
    coordinator/
    macros/
    network/
    node/
    node-service-web/
    utils/
.env.example
```

## Rust Crates

All Rust crates are contained in the `crates` subdirectory.

The actual crate names are prefixed with `pluto-`,
however this is redundant in the folder structure.

### backup (lib)

### cli (bin)

Currently not in use.

### coordinator (bin)

### macros (lib)

### network (lib)

TODO: clean this up.

See [pluto-network](network/overview.md).

### node (lib)

Functionality specific to nodes, excluding backup code itself,
is implemented in this crate.

This allows one definition of node behaviour to be used across multiple
different frontends, such as a [CLI](cli) and gui.

### node-service-web (bin)

This crate implements a user interface for node in the form of a website.
It's split up into a RESTful API implemented in Rust, and a web frontend
using Vue.js.
API implementation is located in `src`, the frontend is in `frontend`.

The API is using warp for routing. Filters (routes) are defined in `src/api/filters.rs`,
and registered in `src/api/mod.rs`. The actual logic for routes is implemented in `src/api/routes`.
We are using a custom macro (`#[reject]`) to allow returning a `Result` with a JSON response.
That makes error handling much nicer when using the `?` operator.

Frontend is made using Vue.js 3, with Vite for building.

### utils (lib)