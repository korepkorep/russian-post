# Russian-post demo

This project demonstrates how to use simple blockchain in Russian-post
based on [Exonum blockchain](https://github.com/exonum/exonum).

Exonum blockchain keeps balances of users and handles secure
transactions between them.

It implements most basic operations:

- Create a new user
- Add funds to the user's balance
- Transfer funds between users
- Issue user's funds
- Prepare some funds for stamping
- Accept preparation transaction

## Install and run

### Using docker

<!-- spell-checker:ignore serhiioryshych -->

Simply run the following command to start the cryptocurrency service on 4 nodes
on the local machine:

```bash
docker run -p 8000-8008:8000-8008 serhiioryshych/exonum-cryptocurrency-advanced-example
```

Ready! Find demo at [http://127.0.0.1:8008](http://127.0.0.1:8008).

Docker will automatically pull image from the repository and
run 4 nodes with public endpoints at `127.0.0.1:8000`, ..., `127.0.0.1:8003`
and private ones at `127.0.0.1:8004`, ..., `127.0.0.1:8007`.

To stop docker container, use `docker stop <container id>` command.

### Manually

#### Getting started

Be sure you installed necessary packages:

- [git](https://git-scm.com/downloads)
- [Node.js with npm](https://nodejs.org/en/download/)
- [Rust compiler](https://rustup.rs/)

#### Install and run

Below you will find a step-by-step guide to starting the cryptocurrency
service on 4 nodes on the local machine.

Build the project:

```sh
cd examples/cryptocurrency-advanced/backend

cargo install
```

Generate template:

<!-- markdownlint-disable MD013 -->

```sh
mkdir example

./exonum-russian-post generate-template example/common.toml --validators-count 4
```

Generate public and secrets keys for each node:

```sh
./exonum-russian-post generate-config example/common.toml  example/pub_1.toml example/sec_1.toml --peer-address 127.0.0.1:6331

./exonum-russian-post generate-config example/common.toml  example/pub_2.toml example/sec_2.toml --peer-address 127.0.0.1:6332

./exonum-russian-post generate-config example/common.toml  example/pub_3.toml example/sec_3.toml --peer-address 127.0.0.1:6333

./exonum-russian-post generate-config example/common.toml  example/pub_4.toml example/sec_4.toml --peer-address 127.0.0.1:6334
```

Finalize configs:

```sh
./exonum-russian-post finalize --public-api-address 0.0.0.0:8200 --private-api-address 0.0.0.0:8091 example/sec_1.toml example/node_1_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-russian-post finalize --public-api-address 0.0.0.0:8201 --private-api-address 0.0.0.0:8092 example/sec_2.toml example/node_2_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-russian-post finalize --public-api-address 0.0.0.0:8202 --private-api-address 0.0.0.0:8093 example/sec_3.toml example/node_3_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

./exonum-russian-post finalize --public-api-address 0.0.0.0:8203 --private-api-address 0.0.0.0:8094 example/sec_4.toml example/node_4_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml
```

Run nodes:

```sh
./exonum-russian-post run --node-config example/node_1_cfg.toml --db-path example/db1 --public-api-address 0.0.0.0:8200

./exonum-russian-post run --node-config example/node_2_cfg.toml --db-path example/db2 --public-api-address 0.0.0.0:8201

./exonum-russian-post run --node-config example/node_3_cfg.toml --db-path example/db3 --public-api-address 0.0.0.0:8202

./exonum-russian-post run --node-config example/node_4_cfg.toml --db-path example/db4 --public-api-address 0.0.0.0:8203
```

<!-- markdownlint-enable MD013 -->
## Configuration
Let's edit Cargo.toml in exonum configuration.
```sh
[workspace]
members = [
    "exonum",
    "testkit",
    "testkit/server",
    "services/configuration",
    "services/time",
    "examples/cryptocurrency",
    "examples/cryptocurrency-advanced/backend",
    "examples/timestamping/backend",
    "examples/russian-post/backend",
]
exclude = [ "exonum/fuzz" ]
```
Then set configuration in russian-post folder in Cargo.toml file.

```sh
[package]
name = "exonum-russian-post"
version = "0.9.0"
authors = ["The Exonum Team <exonum@bitfury.com>"]
homepage = "https://exonum.com/"
repository = "https://github.com/exonum/exonum"
readme = "README.md"
license = "Apache-2.0"
keywords = ["exonum", "blockchain", "example"]
categories = ["rust-patterns", "development-tools::testing"]
description = "Exonum blockchain example implementing a post office."

[badges]
travis-ci = { repository = "exonum/exonum" }
circle-ci = { repository = "exonum/exonum" }

[dependencies]
exonum = { version = "0.9.0", path = "../../../exonum" }
exonum-configuration = { version = "0.9.0", path = "../../../services/configuration" }
exonum-time = { version = "0.9.0", path = "../../../services/time" }
serde = "1.0.0"
serde_derive = "1.0.0"
failure = "=0.1.2"
serde_json = "1.0.24"
chrono = "0.4.5"

[dev-dependencies]
exonum-testkit = { version = "0.9.0", path = "../../../testkit" }
serde_json = "1.0.24"
pretty_assertions = "=0.5.1"
assert_matches = "1.2.0"

[api]
enable_blockchain_explorer = true
```
## License

Cryptocurrency demo is licensed under the Apache License (Version 2.0).
See [LICENSE](LICENSE) for details.
