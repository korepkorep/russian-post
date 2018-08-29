// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! These are tests concerning the API of the cryptocurrency service. See `tx_logic.rs`
//! for tests focused on the business logic of transactions.
//!
//! Note how API tests predominantly use `TestKitApi` to send transactions and make assertions
//! about the storage state.

extern crate exonum;
extern crate exonum_russian_post as cryptocurrency;
extern crate exonum_testkit;
extern crate exonum_time;
#[macro_use]
extern crate serde_json;

use exonum::{
    api::node::public::explorer::TransactionQuery,
    crypto::{self, CryptoHash, Hash, PublicKey, SecretKey}, 
    helpers::Height,
};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};
use exonum_time::{time_provider::MockTimeProvider, TimeService};
// Import data types used in tests from the crate where the service is defined.
use cryptocurrency::{
    api::{WalletInfo, WalletQuery}, transactions::{CreateWallet, Transfer, Issue, MailAcceptance, MailPreparation, Cancellation}, 
    wallet::Wallet,
    CurrencyService,
};

use exonum::encoding::serialize::FromHex;

use std::time::SystemTime;

// Imports shared test constants.
use constants::{ALICE_NAME, BOB_NAME, JOHN_NAME};


mod constants;


/// Check that the wallet creation transaction works when invoked via API.
#[test]
fn test_create_wallet() {
    let (mut testkit, api, _) = create_testkit();
    let user = 1;
    // Create and send a transaction via API
    let (tx, _) = api.create_wallet(BOB_NAME, user);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // Check that the user indeed is persisted by the service.
    let wallet = api.get_wallet(*tx.pub_key()).unwrap();
    println!("create = {}", serde_json::to_string_pretty(&tx).unwrap());
    assert_eq!(wallet.pub_key(), tx.pub_key());
    assert_eq!(wallet.name(), tx.name());
    assert_eq!(wallet.balance(), 100);
}
#[test]
fn test_issue() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, _key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 2);
    
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);

    let tx = Issue :: new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        11,
        0,
        &key_bob,
    );
    println!("issue = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.issue(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 111);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);    

}

#[test]
fn test_acceptance() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 1);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    
    let meta = &String::new();
    let tx = MailPreparation :: new(
        meta,
        tx_alice.pub_key(),
        11,
        0,
        &key_alice,
    );
    println!("preparation = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.preparation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    

    let tx_accept = MailAcceptance :: new(
        tx_bob.pub_key(),
        tx_alice.pub_key(),
        11,
        false,
        1,
        &key_bob,
    );
    println!("acceptance = {}", serde_json::to_string_pretty(&tx_accept).unwrap());
    api.acceptance(&tx_accept);
    testkit.create_block();
    api.assert_tx_status(tx_accept.hash(), &json!({ "type": "success" }));
    // After the transfer transaction is included into a block, we may check new wallet
    // balances.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    
    let meta = &String::new();

    let tx2 = MailPreparation :: new(
        meta,
        tx_alice.pub_key(),
        11,
        2,
        &key_alice,
    );
    println!("preparation_true = {}", serde_json::to_string_pretty(&tx2).unwrap());
    api.preparation(&tx2);
    testkit.create_block();
    api.assert_tx_status(tx2.hash(), &json!({ "type": "success" }));
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    
    //println!("{}", serde_json::to_string_pretty(&tx2).unwrap());
    let tx3 = MailAcceptance :: new(
        tx_bob.pub_key(),
        tx_alice.pub_key(),
        11,
        true,
        3,
        &key_bob,
    );
    println!("acceptance_true = {}", serde_json::to_string_pretty(&tx3).unwrap());
    api.acceptance(&tx3);
    testkit.create_block();
    api.assert_tx_status(tx3.hash(), &json!({ "type": "success" }));
    // After the transfer transaction is included into a block, we may check new wallet
    // balances.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
   // println!("{}", serde_json::to_string_pretty(&tx3).unwrap());
}

#[test]
fn test_preparation() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, _) = api.create_wallet(BOB_NAME, 0);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let meta = &String::new();
    let tx = MailPreparation :: new(
        meta,
        tx_alice.pub_key(),
        11,
        0,
        &key_alice,
    );

    api.preparation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // After the transfer transaction is included into a block, we may check new wallet
    // balances.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}

/// Check that the transfer transaction works as intended.
#[test]
fn test_transfer() {
    // Create 2 wallets.
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, _) = api.create_wallet(BOB_NAME, 0);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));

    // Check that the initial Alice's and Bob's balances persisted by the service.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);

    // Transfer funds by invoking the corresponding API method.
    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transferred amount
        0,  // seed
        &key_alice,
    );
    println!("transfer = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.transfer(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // After the transfer transaction is included into a block, we may check new wallet
    // balances.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 90);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 110);
}

/// Check that a transfer from a non-existing wallet fails as expected.
#[test]
fn test_transfer_from_nonexisting_wallet() {
    let (mut testkit, api, _) = create_testkit();

    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, _) = api.create_wallet(BOB_NAME, 0);
    // Do not commit Alice's transaction, so Alice's wallet does not exist
    // when a transfer occurs.
    testkit.create_block_with_tx_hashes(&[tx_bob.hash()]);

    api.assert_no_wallet(*tx_alice.pub_key());
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);

    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transfer amount
        0,  // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block_with_tx_hashes(&[tx.hash()]);
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 1, "description": "Sender doesn't exist" }),
    );

    // Check that Bob's balance doesn't change.
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}

/// Check that a transfer to a non-existing wallet fails as expected.
#[test]
fn test_transfer_to_nonexisting_wallet() {
    let (mut testkit, api, _) = create_testkit();

    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, _) = api.create_wallet(BOB_NAME, 0);
    // Do not commit Bob's transaction, so Bob's wallet does not exist
    // when a transfer occurs.
    testkit.create_block_with_tx_hashes(&[tx_alice.hash()]);

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    api.assert_no_wallet(*tx_bob.pub_key());

    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        10, // transfer amount
        0,  // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block_with_tx_hashes(&[tx.hash()]);
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 2, "description": "Receiver doesn't exist" }),
    );

    // Check that Alice's balance doesn't change.
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}

/// Check that an overcharge does not lead to changes in sender's and receiver's balances.
#[test]
fn test_transfer_overcharge() {
    let (mut testkit, api, _) = create_testkit();

    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, _) = api.create_wallet(BOB_NAME, 0);
    testkit.create_block();

    // Transfer funds. The transfer amount (110) is more than Alice has (100).
    let tx = Transfer::new(
        tx_alice.pub_key(),
        tx_bob.pub_key(),
        110, // transfer amount
        0,   // seed
        &key_alice,
    );
    api.transfer(&tx);
    testkit.create_block();
    api.assert_tx_status(
        tx.hash(),
        &json!({ "type": "error", "code": 3, "description": "Insufficient currency amount" }),
    );

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}

#[test]
fn test_unknown_wallet_request() {
    let (_testkit, api, _) = create_testkit();

    // Transaction is sent by API, but isn't committed.
    let (tx, _) = api.create_wallet(ALICE_NAME, 0);

    api.assert_no_wallet(*tx.pub_key());
}


#[test]
fn test_cancellation_transfer() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, _) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 0);
    let (tx_john, key_john) = api.create_wallet(JOHN_NAME, 1);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_john.hash(), &json!({ "type": "success" }));
 


    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let tx_transfer = Transfer :: new(
    	tx_bob.pub_key(),
    	tx_alice.pub_key(),
    	60,
    	4,
    	&key_bob,
    );
    println!("transfer for cancel = {}", serde_json::to_string_pretty(&tx_transfer).unwrap());
    api.transfer(&tx_transfer);
    testkit.create_block();
    api.assert_tx_status(tx_transfer.hash(), &json!({ "type": "success" }));

    let tx_hash = &tx_transfer.hash();

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 160);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 40);

    let tx = Cancellation :: new(
        tx_john.pub_key(),
        tx_bob.pub_key(),
        &tx_hash,
        &key_john,
    );
    println!("cancel_for transfer = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.cancellation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // After the transfer transaction is included into a block, we may check new wallet
    // balances.

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}
#[test]
fn test_cancellation_issue() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 2);
    let (tx_john, key_john) = api.create_wallet(JOHN_NAME, 1);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));
    
    //println!("keys_bob = {}, {}", serde_json::to_string_pretty(&tx_bob.pub_key()).unwrap(), serde_json::to_string_pretty(&key_bob).unwrap());
    //println!("keys_alice = {}, {}", serde_json::to_string_pretty(&tx_alice.pub_key()).unwrap(), serde_json::to_string_pretty(&key_alice).unwrap());

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);

    let tx_issue = Issue :: new(
    	tx_alice.pub_key(),
    	tx_bob.pub_key(),
    	60,
    	4,
    	&key_bob,
    );
    println!("issue for cancel = {}", serde_json::to_string_pretty(&tx_issue).unwrap());
    api.issue(&tx_issue);
    testkit.create_block();
    api.assert_tx_status(tx_issue.hash(), &json!({ "type": "success" }));

    let tx_hash = &tx_issue.hash();

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 160);

    let tx = Cancellation :: new(
        tx_john.pub_key(),
        tx_alice.pub_key(),
        &tx_hash,
        &key_john,
    );
    println!("cancel for issue = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.cancellation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    // After the transfer transaction is included into a block, we may check new wallet
    // balances.

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
    let wallet = api.get_wallet(*tx_bob.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}
#[test]
fn test_cancellation_mailpreparation() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 1);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));
 


    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);

    let meta = &String::new();
    let tx_preparation = MailPreparation :: new(
        meta,
        tx_alice.pub_key(),
        11,
        0,
        &key_alice,
    );
    println!("preparation = {}", serde_json::to_string_pretty(&tx_preparation).unwrap());
    api.preparation(&tx_preparation);
    testkit.create_block();
    api.assert_tx_status(tx_preparation.hash(), &json!({ "type": "success" }));
    let tx_hash = &tx_preparation.hash();

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);

    let tx = Cancellation :: new(
        tx_bob.pub_key(),
        tx_alice.pub_key(),
        &tx_hash,
        &key_bob,
    );
    println!("cancellation = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.cancellation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));


    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}
#[test]
fn test_cancellation_mailacceptance() {
    let (mut testkit, api, _) = create_testkit();
    let (tx_alice, key_alice) = api.create_wallet(ALICE_NAME, 0);
    let (tx_bob, key_bob) = api.create_wallet(BOB_NAME, 1);
    testkit.create_block();
    api.assert_tx_status(tx_alice.hash(), &json!({ "type": "success" }));
    api.assert_tx_status(tx_bob.hash(), &json!({ "type": "success" }));
 


    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);


    let meta = &String::new();
    let tx_preparation = MailPreparation :: new(
        meta,
        tx_alice.pub_key(),
        11,
        0,
        &key_alice,
    );
    println!("preparation = {}", serde_json::to_string_pretty(&tx_preparation).unwrap());
    api.preparation(&tx_preparation);
    testkit.create_block();
    api.assert_tx_status(tx_preparation.hash(), &json!({ "type": "success" }));
    
    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);


    let tx_accept = MailAcceptance :: new(
        tx_bob.pub_key(),
        tx_alice.pub_key(),
        11,
        true,
        1,
        &key_bob,
    );
    println!("acceptance = {}", serde_json::to_string_pretty(&tx_accept).unwrap());
    api.acceptance(&tx_accept);
    testkit.create_block();
    api.assert_tx_status(tx_accept.hash(), &json!({ "type": "success" }));

    let tx_hash = &tx_accept.hash();

    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 89);

    let tx = Cancellation :: new(
        tx_bob.pub_key(),
        tx_alice.pub_key(),
        &tx_hash,
        &key_bob,
    );
    println!("cancellation = {}", serde_json::to_string_pretty(&tx).unwrap());
    api.cancellation(&tx);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));


    let wallet = api.get_wallet(*tx_alice.pub_key()).unwrap();
    assert_eq!(wallet.balance(), 100);
}
/// Wrapper for the cryptocurrency service API allowing to easily use it
/// (compared to `TestKitApi` calls).
struct CryptocurrencyApi {
    pub inner: TestKitApi,
}

impl CryptocurrencyApi {
    /// Generates a wallet creation transaction with a random key pair, sends it over HTTP,
    /// and checks the synchronous result (i.e., the hash of the transaction returned
    /// within the response).
    /// Note that the transaction is not immediately added to the blockchain, but rather is put
    /// to the pool of unconfirmed transactions.
    fn create_wallet(&self, name: &str, user: u64) -> (CreateWallet, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        // Create a pre-signed transaction
        let tx = CreateWallet::new(&pubkey, name, user, &key);
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn get_wallet(&self, pub_key: PublicKey) -> Option<Wallet> {
        let wallet_info = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&WalletQuery { pub_key })
            .get::<WalletInfo>("v1/wallets/info")
            .unwrap();

        let to_wallet = wallet_info.wallet_proof.to_wallet.check().unwrap();
        to_wallet
            .all_entries()
            .iter()
            .find(|(ref k, _)| **k == pub_key)
            .and_then(|tuple| tuple.1)
            .cloned()
    }

    fn preparation(&self, tx: &MailPreparation) {
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
    }

    fn acceptance(&self, tx: &MailAcceptance) {
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));        
    }
    
    /// Sends a transfer transaction over HTTP and checks the synchronous result.
    fn transfer(&self, tx: &Transfer) {
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
    }

    fn issue(&self, tx: &Issue) {
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
    }

    fn cancellation(&self, tx: &Cancellation) {
    	let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&tx)
            .post("v1/wallets/transaction")
            .unwrap();

        assert_eq!(tx_info, json!({ "tx_hash": tx.hash()}));
    }

    /// Asserts that a wallet with the specified public key is not known to the blockchain.
    fn assert_no_wallet(&self, pub_key: PublicKey) {
        let wallet_info: WalletInfo = self.inner
            .public(ApiKind::Service("cryptocurrency"))
            .query(&WalletQuery { pub_key })
            .get("v1/wallets/info")
            .unwrap();

        let to_wallet = wallet_info.wallet_proof.to_wallet.check().unwrap();
        assert!(
            to_wallet
                .missing_keys()
                .iter()
                .find(|v| ***v == pub_key)
                .is_some()
        )
    }

    /// Asserts that the transaction with the given hash has a specified status.
    fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self.inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();
        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }
}

/// Creates a testkit together with the API wrapper defined above.
fn create_testkit() -> (TestKit, CryptocurrencyApi, MockTimeProvider) {
	let mock_provider = MockTimeProvider::new(SystemTime::now().into());
    let mut testkit = TestKitBuilder::validator()
        .with_service(CurrencyService)
        .with_service(TimeService::with_provider(mock_provider.clone()))
        .create();
    let api = CryptocurrencyApi {
        inner: testkit.api(),
    };
    testkit.create_blocks_until(Height(2)); 
    (testkit, api, mock_provider)
}
