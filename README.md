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

## Wallet

Firstly, to describe user interaction we need to add wallet object, because the main feature is exchange of tokens

```sh
use exonum::crypto::{Hash, PublicKey};

encoding_struct! {
    /// Wallet information stored in the database.
    struct Wallet {
        pub_key:            &PublicKey,
        name:               &str,
        balance:            u64,
        history_len:        u64,
        history_hash:       &Hash,
        freezed_balance:    u64,
    }
}
``` 
``` pub_key``` is a field of the wallet holder, ```  name ``` is a field of the wallet name,
``` balance``` is a field of the wallet balance, ``` freezed_balance``` is a field of the wallet that cannot be spent (Used in MailPreparation Transaction). Two other fields show the information about history of the wallet.

```sh
impl Wallet {
    /// Returns a copy of this wallet with updated balance.
    pub fn set_balance(self, balance: u64, history_hash: &Hash, freezed_balance: u64) -> Self {
        Self::new(
            self.pub_key(),
            self.name(),
            balance,
            self.history_len() + 1,
            history_hash,
            freezed_balance,
        )
    }
}
```
Here we can set balance of the wallet.

##Transactions

Now we have wallet. Let's define transactions, they will describe the interaction between users of the blockchain.
First, we are using some imports, that will help us to use necessary files(DataBase, Macroses, DataTypes and etc.).
```sh
extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize, Deserializer, Serializer};

use chrono::{DateTime, Utc};

use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey, Hash};
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum::storage::StorageValue;
use exonum::messages::RawMessage;
use exonum::storage::Snapshot;
use exonum_time::schema::TimeSchema;

use POST_SERVICE_ID;
use schema::{CurrencySchema, TimestampEntry};
```
Second, defining transaction structs.

```sh
transactions! {
    pub WalletTransactions {
        const SERVICE_ID = POST_SERVICE_ID;

        struct Transfer {
            from:    &PublicKey,
            to:      &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        struct Issue {
            pub_key:  &PublicKey,
            issuer_key: &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        struct CreateWallet {
            pub_key: &PublicKey,
            name:    &str,
        }

        struct MailPreparation {
            meta: &str,
            pub_key: &PublicKey,
            amount: u64,
            seed: u64,
        }

        struct MailAcceptance {
            pub_key: &PublicKey,
            sender_key: &PublicKey,
            amount: u64,
            accept:  bool,
            seed: u64,
        }
        
        struct Cancellation {
            pub_key: &PublicKey,
            sender: &PublicKey,
            tx_hash: &Hash,
        }
    }
}
```

#Transfer

Transfer transaction has 4 fields. The first one is ```from```. This field contains sender's public key.
The second one is  ```to```. This field contains recipient's public key.
The third one is ```amount```. It contains information "How many funds we are going to transfer".
The last one is ```seed```. This field is special, because we need it, to avoid repetition of the equal transactions.

#Issue

Issue transaction has 4 fields. The first one is ```pub_key```. This field contains public key of the wallet holder, whose wallet balance
should be increased.
The second one is  ```issuer_key```. This field contains public key of the issuer.
The third one is ```amount```. It contains information "How many funds we are going to issue".
The last one is ```seed```. This field is special, because we need it, to avoid repetition of the equal transactions.

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
## License

Cryptocurrency demo is licensed under the Apache License (Version 2.0).
See [LICENSE](LICENSE) for details.
