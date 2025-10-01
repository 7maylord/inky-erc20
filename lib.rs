#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod inky_bank {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct InkyBank {
        owner: AccountId,
        total_supply: u128,
        balances: Mapping<AccountId, u128>,
        allowances: Mapping<(AccountId, AccountId), u128>,
        paused: bool,
        blacklist: Mapping<AccountId, bool>,
    }

    /// Events
    #[ink(event)]
    pub struct Minted {
        #[ink(topic)]
        to: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Paused {
        #[ink(topic)]
        paused: bool,
    }

    #[ink(event)]
    pub struct Burned {
        #[ink(topic)]
        from: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Blacklisted {
        #[ink(topic)]
        account: AccountId,
        status: bool,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        NotOwner,
        ZeroAmount,
        ContractPaused,
        AccountBlacklisted,
        InsufficientAllowance,
        InvalidBatchOperation,
    }

    /// Result type for our contract functions
    pub type Result<T> = core::result::Result<T, Error>;

    impl Default for InkyBank {
        fn default() -> Self {
            Self::new()
        }
    }

    impl InkyBank {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                owner: caller,
                total_supply: 0,
                balances: Mapping::default(),
                allowances: Mapping::default(),
                paused: false,
                blacklist: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if self.blacklist.get(to).unwrap_or(false) {
                return Err(Error::AccountBlacklisted);
            }

            if amount == 0 {
                return Err(Error::ZeroAmount);
            }

            let current_balance = self.balance_of(to);
            let new_balance = current_balance.saturating_add(amount);
            self.balances.insert(to, &new_balance);

            self.total_supply = self.total_supply.saturating_add(amount);

            self.env().emit_event(Minted { to, amount });

            Ok(())
        }

       

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let from = self.env().caller();

            if amount == 0 {
                return Err(Error::ZeroAmount);
            }

            let from_balance = self.balance_of(from);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let new_from_balance = from_balance.saturating_sub(amount);
            self.balances.insert(from, &new_from_balance);

            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance.saturating_add(amount);
            self.balances.insert(to, &new_to_balance);

            self.env().emit_event(Transfer { from, to, amount });

            Ok(())
        }

       

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: u128) -> Result<()> {
            let owner = self.env().caller();

            self.allowances.insert((owner, spender), &amount);

            self.env().emit_event(Approval { owner, spender, amount });
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if amount == 0 {
                return Err(Error::ZeroAmount);
            }

            if self.allowances.get((from, caller)).unwrap_or(0) < amount {
                return Err(Error::InsufficientAllowance);
            }

            let from_balance = self.balance_of(from);
            if from_balance < amount {
                return Err(Error::InsufficientBalance);
            }

            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance.saturating_add(amount);
            self.balances.insert(to, &new_to_balance);

            let from_balance = from_balance.saturating_sub(amount);
            self.balances.insert(from, &from_balance);
            
            self.env().emit_event(Transfer { from, to, amount });
            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        #[ink(message)]
        pub fn batch_transfer(&mut self, recipients: Vec<AccountId>, amounts: Vec<u128>) -> Result<()> {
            let caller = self.env().caller();

            if self.paused {
                return Err(Error::ContractPaused);
            }

            if recipients.len() != amounts.len() {
                return Err(Error::InvalidBatchOperation);
            }

            let total_amount: u128 = amounts.iter().sum();

            if total_amount > self.balance_of(caller) {
                return Err(Error::InsufficientBalance);
            }
            
            for (i, recipient) in recipients.iter().enumerate() {
                self.transfer(*recipient, amounts[i])?;
            }

            Ok(())
            
        }

        #[ink(message)]
        pub fn toggle_pause(&mut self, paused: bool) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            self.paused = paused;
            self.env().emit_event(Paused { paused });
            Ok(())
        }


        #[ink(message)]
        pub fn burn(&mut self, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            if amount == 0 {
                return Err(Error::ZeroAmount);
            }

            let balance = self.balance_of(caller);
            if balance < amount {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(caller, &balance.saturating_sub(amount));
            self.total_supply = self.total_supply.saturating_sub(amount);

            self.env().emit_event(Burned { from: caller, amount });
            Ok(())
        }

        #[ink(message)]
        pub fn toggle_blacklist(&mut self, account: AccountId, status: bool) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            self.blacklist.insert(account, &status);
            self.env().emit_event(Blacklisted { account, status });
            Ok(())
        }
        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

    }
}