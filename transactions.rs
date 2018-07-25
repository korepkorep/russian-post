
transactions! {
    pub WalletTransactions {
        const SERVICE_ID = CRYPTOCURRENCY_SERVICE_ID;

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
            amount:  u64,
            seed:    u64,
        }

        /// Create wallet with the given `name`.
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
            sender: &PublicKey,
            pub_key: &PublicKey,
            amount: u64,
            accept:  bool,
            seed: u64,
        }
//INSERT one type
        struct Cancellation {
            pub_key: &PublicKey,
            name: &str,
        }
    }
}

impl Transaction for Issue {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if let Some(wallet) = schema.wallet(pub_key) {
            let amount = self.amount();
            schema.increase_wallet_balance(wallet, amount, &hash);
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
        let mut schema = CurrencySchema::new(fork);
        let from = self.from();
        let to = self.to();
        let hash = self.hash();
        let amount = self.amount();

        let sender = schema.wallet(from).ok_or(Error :: SenderNotFound)?;

        let receiver = schema.wallet(to).ok_or(Error :: ReceiverNotFound)?;

        if sender.balance() < amount {
            Err(Error::InsufficientCurrencyAmount)?;

        }

        schema.decrease_wallet_balance(sender, amount, &hash);
        schema.increase_wallet_balance(receiver, amount, &hash);

        Ok(())
    }
}

impl Transaction for CreateWallet {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema::new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if schema.wallet(pub_key).is_none(){
            let name = self.name();
            schema.create_wallet(pub_key, name, &hash);
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
        Ok(())
    }
}


impl Transaction for MailAcceptance {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }



    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema :: new(fork);
        let sender_key = self.sender();

        let hash = self.hash();
        let sender = schema.wallet(sender_key).ok_or(Error :: SenderNotFound)?;
        let freezed_balance = 0;
        schema.decrease_wallet_balance(sender, freezed_balance, &hash, freezed_balance);
        Ok(())

    }
}

impl Transaction for Cancellation {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> ExecutionResult {
        let mut schema = CurrencySchema :: new(fork);
        let pub_key = self.pub_key();
        let hash = self.hash();

        if !(schema.wallet(pub_key).is_none()){
            let name = self.name();
            schema.delete_wallet(pub_key, name, &hash);
            Ok(())
        } else {
//Write/Add Error in Errors
            Err(Error::WalletNotFound)?
        }
    }
}