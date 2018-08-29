
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

#![ allow( bare_trait_objects ) ]

extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize, Deserializer, Serializer};


use exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use exonum::crypto::{CryptoHash, PublicKey, Hash};
use exonum::messages::Message;
use exonum::storage::Fork;
use exonum_time::schema::TimeSchema;

use POST_SERVICE_ID;
use schema::{CurrencySchema, TimestampEntry};

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

    #[fail(display = "Pubkey doesn`t belong to inspector")]
    NotInspector = 5,

    #[fail(display = "Pubkey doesn`t belong to issuer")]
    NotIssuer = 6,
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

        /// Transfer `amount` of the currency from one wallet to another.
        struct Transfer {
            from:    &PublicKey,
            to:      &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        /// Issue `amount` of the currency to the `wallet`.
        struct Issue {
            pub_key:  &PublicKey,
            issuer_key: &PublicKey,
            amount:  u64,
            seed:    u64,
        }

        /// Create wallet with the given `name`. 1 - inspector, 0 - user, 2 - issuer
        struct CreateWallet {
            pub_key: &PublicKey,
            name:    &str,
            user_type: u64,
        }

        /// Prepare tokens for stamping 
        struct MailPreparation {
            meta: &str,
            pub_key: &PublicKey,
            amount: u64,
            seed: u64,
        }

        /// Accept or reject prepared tokens
        struct MailAcceptance {
            pub_key: &PublicKey,
            sender_key: &PublicKey,
            amount: u64,
            accept:  bool,
            seed: u64,
        }
        
        /// Cancel particular transaction
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
        if !schema.issuers().contains(self.issuer_key()) {
        	Err(Error::NotIssuer)?
        }
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
            schema.add_inspector(pub_key, self.user_type());
            schema.add_issuer(pub_key, self.user_type());
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
        if !schema.inspectors().contains(self.pub_key()) {
        	Err(Error::NotInspector)?
        }
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
        let tx_hash = self.tx_hash();
        let hash = self.hash();
        if !schema.inspectors().contains(self.pub_key()) {
        	Err(Error::NotInspector)?
        }
        let tx_time = schema.timestamps().get(&tx_hash).unwrap();
        if time.timestamp() - tx_time < n {
            let raw_tx = match schema.transactions().get(&tx_hash) {
            	Some(x) => x,
            	None => panic!("Transaction not found!"),
            };
            let id = raw_tx.message_type();
            match id {
                0 => {
                	let transaction: Transfer = Message::from_raw(raw_tx.clone()).unwrap();
	                let from = transaction.from();
	                let to = transaction.to();
	                let amount = transaction.amount();
	                let wallet_from = schema.wallet(&from).ok_or(Error :: SenderNotFound)?;
	                let wallet_to = schema.wallet(to).ok_or(Error :: ReceiverNotFound)?;
	                schema.decrease_wallet_balance(wallet_to, amount, &tx_hash, 0);
	                schema.increase_wallet_balance(wallet_from, amount, &tx_hash, 0);
	            },
	            1 => {
	            	let transaction: Issue = Message::from_raw(raw_tx.clone()).unwrap();
	                let pub_key = transaction.pub_key();
	                let amount = transaction.amount();
	                let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
	                schema.decrease_wallet_balance(sender, amount, &tx_hash, 0);
	              
	            },
	            3 => {
	                let transaction: MailPreparation = Message::from_raw(raw_tx.clone()).unwrap();
	                let pub_key = transaction.pub_key();
	                let amount = transaction.amount();
	                let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
	                schema.increase_wallet_balance(sender, amount, &hash, 0);
	               
	            },
                4 => {
                	let transaction: MailAcceptance = Message::from_raw(raw_tx.clone()).unwrap();
                	if transaction.accept() {
                    	let pub_key = transaction.sender_key();
                    	let amount = transaction.amount();
                    	let sender = schema.wallet(&pub_key).ok_or(Error :: ReceiverNotFound)?;
                    	schema.increase_wallet_balance(sender, amount, &hash, 0);
                    }
                    
                },
                _ => panic!("Transaction is not defined"),
       		};
       		let entry = TimestampEntry::new(&self.hash(), time);
        	schema.add_timestamp(entry);
       	} else {
       		Err(Error::Timeisup)?;
       	}
        Ok(())
    }
}