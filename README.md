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

## Transactions

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
Second, defining transaction structs and define errors.

```sh
/// Error codes emitted by wallet transactions during execution.
#[derive(Debug, Fail)]
#[repr(u8)]
pub enum Error {
    /// Wallet already exists.
    ///
    /// Can be emitted by `CreateWallet`.
    #[fail(display = "Wallet already exists")]
    WalletAlreadyExists = 0,

    /// Sender doesn't exist.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Sender doesn't exist")]
    SenderNotFound = 1,

    /// Receiver doesn't exist.
    ///
    /// Can be emitted by `Transfer` or `Issue`.
    #[fail(display = "Receiver doesn't exist")]
    ReceiverNotFound = 2,

    /// Insufficient currency amount.
    ///
    /// Can be emitted by `Transfer`.
    #[fail(display = "Insufficient currency amount")]
    InsufficientCurrencyAmount = 3,

    #[fail(display = "Time is up")]
    Timeisup = 4,
}

impl From<Error> for ExecutionError {
    fn from(value: Error) -> ExecutionError {
        let description = format!("{}", value);
        ExecutionError::with_description(value as u8, description)
    }
}

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


impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.issuer_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();

        if let Some(wallet) = schema.wallet(pub_key) {
            let amount = self.amount();
            schema.increase_wallet_balance(wallet, amount, &self.hash(), 0);

            let entry = TimestampEntry::new(&self.hash(), time.unwrap());
            schema.add_timestamp(entry);

            Ok(())
        } else {
            Err(Error::ReceiverNotFound)?
        }

    }

}


impl Transaction for Transfer {
    fn verify(&self) -> bool {
        (self.from() != self.to()) && self.verify_signature(self.from())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        
        let mut schema = CurrencySchema::new(fork);
        let from = self.from();
        let to = self.to();
        let hash = self.hash();
        let amount = self.amount();
        let freezed_balance = 0;
        let sender = schema.wallet(from).ok_or(Error :: SenderNotFound)?;

        let receiver = schema.wallet(to).ok_or(Error :: ReceiverNotFound)?;

        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?;

        }

        schema.decrease_wallet_balance(sender, amount, &hash, freezed_balance);
        schema.increase_wallet_balance(receiver, amount, &hash, freezed_balance);
        
        let entry = TimestampEntry::new(&self.hash(), time.unwrap());
        schema.add_timestamp(entry);
        Ok(())
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();



        if schema.wallet(pub_key).is_none(){
            let name = self.name();
            let freezed_balance = 0;
            schema.create_wallet(pub_key, name, &hash, freezed_balance);

            let entry = TimestampEntry::new(&self.hash(), time.unwrap());
            schema.add_timestamp(entry);

            Ok(())
        } else {
            Err(Error::WalletAlreadyExists)?
        } 
    } 
}


impl Transaction for MailPreparation {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();
        let amount = self.amount();
        let hash = self.hash();
        let sender = schema.wallet(pub_key).ok_or(Error :: SenderNotFound)?;
        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?;
        }
        // freeze_wallet_balance rrealize
        schema.decrease_wallet_balance(sender, amount, &hash, amount);
        let entry = TimestampEntry::new(&self.hash(), time.unwrap());
        schema.add_timestamp(entry);
        Ok(())
    }
}


impl Transaction for MailAcceptance {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }



    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let time = TimeSchema::new(&fork)
            .time()
            .get();
        let mut schema = CurrencySchema :: new(fork);
        let sender_key = self.sender_key();
        let accept = self.accept();
        let hash = self.hash();
        let sender = schema.wallet(sender_key).ok_or(Error :: SenderNotFound)?;
        if accept {
            let freezed_balance = 0;
            schema.decrease_wallet_balance(sender, freezed_balance, &hash, freezed_balance);
        } else {
            let amount = sender.freezed_balance();
            let freezed_balance = 0;
            schema.increase_wallet_balance(sender, amount, &hash, freezed_balance);
        }
        let entry = TimestampEntry::new(&self.hash(), time.unwrap());
        schema.add_timestamp(entry);
        Ok(())

    }
}

impl Transaction for Cancellation {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let n = 100000;
        let time = TimeSchema::new(&fork)
            .time()
            .get()
            .unwrap();

        let mut schema = CurrencySchema :: new(fork);

        let sender_key = self.sender();
        let tx_hash = self.tx_hash();
        let hash = self.hash();
        let tx_time = schema.timestamps().get(&tx_hash).unwrap();
        if time.timestamp() - tx_time < n {
            let raw_tx = schema.transactions().get(&tx_hash).unwrap();
            if raw_tx.message_type() == 0 { //Transfer
                let transaction: Transfer = Message::from_raw(raw_tx.clone()).unwrap();
                let from = transaction.from();
                let to = transaction.to();
                let amount = transaction.amount();
                let wallet_from = schema.wallet(&from).ok_or(Error :: SenderNotFound)?;
                let wallet_to = schema.wallet(to).ok_or(Error :: ReceiverNotFound)?;
                schema.decrease_wallet_balance(wallet_to, amount, &tx_hash, 0);
                schema.increase_wallet_balance(wallet_from, amount, &tx_hash, 0);
            } else if raw_tx.message_type() == 1 { //issue
                let transaction: Issue = Message::from_raw(raw_tx.clone()).unwrap();
                let pub_key = transaction.pub_key();
                let amount = transaction.amount();
                let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
                schema.decrease_wallet_balance(sender, amount, &tx_hash, 0);
            } else if raw_tx.message_type() == 3 { //MailPreparation
                let transaction: MailPreparation = Message::from_raw(raw_tx.clone()).unwrap();
                let pub_key = transaction.pub_key();
                let amount = transaction.amount();
                let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
                schema.increase_wallet_balance(sender, amount, &hash, 0);
            } else if raw_tx.message_type() == 4 { //MailAcceptance
                let transaction: MailAcceptance = Message::from_raw(raw_tx.clone()).unwrap();
                if transaction.accept() {
                    let pub_key = transaction.sender_key();
                    let amount = transaction.amount();
                    let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
                    schema.increase_wallet_balance(sender, amount, &hash, 0);
                }
            }
        } else {
            Err(Error::Timeisup)?;
        }
        
        
        let entry = TimestampEntry::new(&self.hash(), time);
        schema.add_timestamp(entry);
        Ok(())

    }

}
```

#### Transfer

Transfer transaction has 4 fields. The first one is ```from```. This field contains sender's public key.
The second one is  ```to```. This field contains recipient's public key.
The third one is ```amount```. It contains information "How many funds we are going to transfer".
The last one is ```seed```. This field is special, because we need it, to avoid repetition of the equal transactions.

#### Issue

Issue transaction has 4 fields. The first one is ```pub_key```. This field contains public key of the wallet holder, whose wallet balance
should be increased.
The second one is  ```issuer_key```. This field contains public key of the issuer.
The third one is ```amount```. It contains information "How many funds we are going to issue".
The last one is ```seed```. This field is special, because we need it, to avoid repetition of the equal transactions.

#### Create Wallet

Create Wallet transaction has 2 fields. The first one is ```pub_key```. This field contains public key of the wallet creator.
The second one is ```name```. This field contains the name of the wallet.

#### Mail Preparation

This kind of transactions describes proccess of the token stamping. If entity wants to stamp some tokens, he need this transaction. Mail Preparation transaction has 4 fields. The first one is ```meta```. It contains information about stamping. F.e. "I would like to stamp 3000 tokens". The second one is ```pub_key```. It contains information about entity public key. The third one is ```amount```. This field is about the number of tokens that should be stamped. The last field is seed field.

#### Mail Acceptance

Only inspectors can accept or reject Mail Preparation transaction. To accept/reject Mail Preparation, they use Mail Acceptance Transaction.
Mail Acceptance transaction has 5 fields. The first one is ```pub_key```. It contains information about inspector public key. The second one is ```sender_key```. It contains information about person public key who wants to stamp tokens. The third one is ```amount```. This field is about the number of tokens that should be stamped. The fourth one is ```accept```. This field is about inspector's decision (Accept or reject). The last field is seed field.

#### Cancellation

This kind of transaction needs three fields. The first one is ```pub_key```. This field contains public key of the inspector.
The second one is ```sender_key```. This field contains the public key of the transaction creator. The last one is ```tx_hash```. This field contains hash of transaction that should be cancelled.

## Api

To work with our blockchain we need to proccess requests. Also we want to get/post some requests and see what's going on (transactions, wallet info and etc.).
```sh
use exonum::{
    api::{self, ServiceApiBuilder, ServiceApiState},
    blockchain::{self, BlockProof, Transaction, TransactionSet}, crypto::{Hash, PublicKey},
    helpers::Height, node::TransactionSend, storage::{ListProof, MapProof},
};

use transactions::WalletTransactions;
use wallet::Wallet;
use {CurrencySchema, POST_SERVICE_ID};

/// The structure describes the query parameters for the `get_wallet` endpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WalletQuery {
    /// Public key of the queried wallet.
    pub pub_key: PublicKey,
}

/// The structure returned by the REST API.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Hash of the transaction.
    pub tx_hash: Hash,
}

/// Proof of existence for specific wallet.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletProof {
    /// Proof to the whole database table.
    pub to_table: MapProof<Hash, Hash>,
    /// Proof to the specific wallet in this table.
    pub to_wallet: MapProof<PublicKey, Wallet>,
}

/// Wallet history.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletHistory {
    pub proof: ListProof<Hash>,
    pub transactions: Vec<WalletTransactions>,
}

/// Wallet information.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub block_proof: BlockProof,
    pub wallet_proof: WalletProof,
    pub wallet_history: Option<WalletHistory>,
}

// TODO: Add documentation. (ECR-1638)
/// Public service API description.
#[derive(Debug, Clone, Copy)]
pub struct CryptocurrencyApi;

impl CryptocurrencyApi {
    pub fn wallet_info(state: &ServiceApiState, query: WalletQuery) -> api::Result<WalletInfo> {
        let snapshot = state.snapshot();
        let general_schema = blockchain::Schema::new(&snapshot);
        let currency_schema = CurrencySchema::new(&snapshot);

        let max_height = general_schema.block_hashes_by_height().len() - 1;

        let block_proof = general_schema
            .block_and_precommits(Height(max_height))
            .unwrap();

        let to_table: MapProof<Hash, Hash> =
            general_schema.get_proof_to_service_table(POST_SERVICE_ID, 0);

        let to_wallet: MapProof<PublicKey, Wallet> =
            currency_schema.wallets().get_proof(query.pub_key);

        let wallet_proof = WalletProof {
            to_table,
            to_wallet,
        };

        let wallet = currency_schema.wallet(&query.pub_key);

        let wallet_history = wallet.map(|_| {
            let history = currency_schema.wallet_history(&query.pub_key);
            let proof = history.get_range_proof(0, history.len());

            let transactions: Vec<WalletTransactions> = history
                .iter()
                .map(|record| general_schema.transactions().get(&record).unwrap())
                .map(|raw| WalletTransactions::tx_from_raw(raw).unwrap())
                .collect::<Vec<_>>();

            WalletHistory {
                proof,
                transactions,
            }
        });

        Ok(WalletInfo {
            block_proof,
            wallet_proof,
            wallet_history,
        })
    }

    pub fn post_transaction(
        state: &ServiceApiState,
        query: WalletTransactions,
    ) -> api::Result<TransactionResponse> {
        let transaction: Box<dyn Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { tx_hash })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/wallets/info", Self::wallet_info)
            .endpoint_mut("v1/wallets/transaction", Self::post_transaction);
    }
}
```

## Schema

Now, we have api, wallet, transactions. To combine this files we need the storage. Let's do it.

```sh
use exonum::{
    crypto::{Hash, PublicKey}, storage::{Fork, ProofListIndex, ProofMapIndex, Snapshot, MapIndex},
    messages::{RawMessage},
};

use chrono::{DateTime, Utc};

use wallet::Wallet;
use INITIAL_BALANCE;


encoding_struct! {
    /// Timestamp entry.
    struct TimestampEntry {

        /// Hash of transaction.
        tx_hash: &Hash,

        /// Timestamp time.
        time: DateTime<Utc>,
    }
}



/// Database schema for the cryptocurrency.
#[derive(Debug)]
pub struct CurrencySchema<T> {
    view: T,
}

impl<T> AsMut<T> for CurrencySchema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> CurrencySchema<T>
where
    T: AsRef<dyn Snapshot>,
{
    /// Constructs schema from the database view.
    pub fn new(view: T) -> Self {
        CurrencySchema { view }
    }

    /// Returns `MerklePatriciaTable` with wallets.
    pub fn wallets(&self) -> ProofMapIndex<&T, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &self.view)
    }

    /// Returns history of the wallet with the given public key.
    pub fn wallet_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &self.view)
    }

    /// Returns wallet for the given public key.
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    /// Returns state hash of service database.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.wallets().merkle_root()]
    }

    /// Returns table that represents a map from transaction hash into raw transaction message.
    pub fn transactions(&self) -> MapIndex<&T, Hash, RawMessage> {
        MapIndex::new("core.transactions", &self.view)
    }

    /// Returns the `ProofMapIndex` of timestamps.
    pub fn timestamps(&self) -> ProofMapIndex<&T, Hash, i64> {
        ProofMapIndex::new("cryptocurrency.timestamps", &self.view)
    }

    /// Returns the state hash of the timestamping service.
    pub fn state_hash_timestamps(&self) -> Vec<Hash> {
        vec![self.timestamps().merkle_root()]
    }
}

/// Implementation of mutable methods.
impl<'a> CurrencySchema<&'a mut Fork> {
    /// Returns mutable `MerklePatriciaTable` with wallets.
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &mut self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn wallet_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::new_in_family("cryptocurrency.wallet_history", public_key, &mut self.view)
    }

    /// Increase balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn increase_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash, freezed_balance: u64) {
        let wallet = {
            let mut history = self.wallet_history_mut(wallet.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance();
            wallet.set_balance(balance + amount, &history_hash, freezed_balance)
        };
        self.wallets_mut().put(wallet.pub_key(), wallet.clone());
    }

    /// Decrease balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn decrease_wallet_balance(&mut self, wallet: Wallet, amount: u64, transaction: &Hash, freezed_balance: u64) {
        let wallet = {
            let mut history = self.wallet_history_mut(wallet.pub_key());
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let balance = wallet.balance();
            wallet.set_balance(balance - amount, &history_hash, freezed_balance)
        };
        self.wallets_mut().put(wallet.pub_key(), wallet.clone());
    }

    /// Create new wallet and append first record to its history.
    pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash, freezed_balance: u64) {
        let wallet = {
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);
            let history_hash = history.merkle_root();
            let freezed_balance = 0;
            Wallet::new(key, name, INITIAL_BALANCE, history.len(), &history_hash, freezed_balance)
        };
        self.wallets_mut().put(key, wallet);
    }

    /// Returns mut table that represents a map from transaction hash into raw transaction message.
    pub fn transactions_mut(&mut self) -> MapIndex<&mut Fork, Hash, RawMessage> {
        MapIndex::new("core.transactions", &mut self.view)
    }

    /// Returns the mutable `ProofMapIndex` of timestamps.
    pub fn timestamps_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, i64> {
        ProofMapIndex::new("cryptocurrency.timestamps", &mut self.view)
    }

    /// Adds the timestamp entry to the database.
    pub fn add_timestamp(&mut self, timestamp_entry: TimestampEntry) {
        let tx_hash = timestamp_entry.tx_hash();
        let time = timestamp_entry.time();

        // Check that timestamp with given content_hash does not exist.
        if self.timestamps().contains(tx_hash) {
            return;
        }

        // Add timestamp
        self.timestamps_mut().put(tx_hash, time.timestamp());
    }
}
```
Now, we described all methods that we need in our database. So, now we can extract neccessary transactions, wallets and etc.
Almost everything is ready to launch the blockchain.

## Main

Main file is all we need to launch.

```sh
extern crate exonum;
extern crate exonum_configuration;
extern crate exonum_russian_post;
extern crate exonum_time;

use exonum::helpers::{self, fabric::NodeBuilder};
use exonum_configuration as configuration;
use exonum_russian_post as cryptocurrency;
use exonum_time::TimeServiceFactory;


fn main() {
    exonum::crypto::init();
    helpers::init_logger().unwrap();

    let node = NodeBuilder::new()
        .with_service(Box::new(configuration::ServiceFactory))
        .with_service(Box::new(TimeServiceFactory))
        .with_service(Box::new(cryptocurrency::ServiceFactory));
    node.run();
}
```

Ok, everything is ready. Let's run our project.

#### Install and run

Below you will find a step-by-step guide to starting the post-office
service on 4 nodes on the local machine.

Build the project:

```sh
cd examples/russian-post/backend

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

## Interaction

To interact with blockchain we will use ```.json``` files. There are the examples of Mail Preparation and Mail Acceptance transactions below.

##### Mail Preparation

```sh
{
  "body": {
    "amount": "11",
    "meta": "",
    "pub_key": "ae5a9a90348ca866e3d6f878c35f2130f9b6d4d8daabfadc7ff15c65c1994bf5",
    "seed": "0"
  },
  "message_id": 3,
  "protocol_version": 0,
  "service_id": 128,
  "signature": "89234fdb23065155e82e120f8bac5e21726107c968330e6e60ab6b9ff9b8a48a71b0fc0afab90118056a7a3510243db980179e53fe8f564ebc81608f1f87270e"
}
```

##### Mail Acceptance

```sh
{
  "body": {
    "accept": false,
    "amount": "11",
    "pub_key": "5f3eb2f672baab17a9a06de555672a019630818a8167b994066bc90d4d7efa77",
    "seed": "1",
    "sender_key": "ae5a9a90348ca866e3d6f878c35f2130f9b6d4d8daabfadc7ff15c65c1994bf5"
  },
  "message_id": 4,
  "protocol_version": 0,
  "service_id": 128,
  "signature": "3a6310285c4b82f1e14205018e64b28a4cb4fa79261a7e60483f0510b524752775ca12853835bc27b124819574f8351874d7ea95996b6211fe33ce27b76ccb01"
}
```
To send requests you may use Postman or curl.

<!-- markdownlint-enable MD013 -->
## License

Cryptocurrency demo is licensed under the Apache License (Version 2.0).
See [LICENSE](LICENSE) for details.
