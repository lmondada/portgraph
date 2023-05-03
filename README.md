portgraph
=========

[![build_status][]](https://github.com/CQCL/portgraph/actions)
[![crates][]](https://crates.io/crates/portgraph)
[![msrv][]](https://github.com/CQCL/portgraph)

Data structure library for directed graphs with first-level ports. Includes
secondary data structures for node and port weights, and node hierarchies.

Please read the [API documentation here][].

## Features

-   `pyo3`: Enable Python bindings via pyo3.
-   `serde`: Enable serialization and deserialization via serde.

## Recent Changes

See [RELEASES][] for a list of changes. The minimum supported rust
version will only change on major releases.

## License

This project is licensed under Apache License, Version 2.0 ([LICENSE][] or http://www.apache.org/licenses/LICENSE-2.0).

  [API documentation here]: https://docs.rs/portgraph/
  [build_status]: https://github.com/CQCL/portgraph/workflows/Continuous%20integration/badge.svg?branch=main
  [crates]: https://img.shields.io/crates/v/portgraph
  [LICENSE]: LICENCE
  [msrv]: https://img.shields.io/badge/rust-1.69.0%2B-blue.svg?maxAge=3600
  [RELEASES]: RELEASES.rst