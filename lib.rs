#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod inky_bank {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct InkyBank {
        owner: AccountId,
        total_supply: u128,
        balances: Mapping<AccountId, u128>,
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

    /// Errors
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        NotOwner,
        ZeroAmount,
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
            }
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, amount: u128) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
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
        pub fn balance_of(&self, account: AccountId) -> u128 {
            self.balances.get(account).unwrap_or(0)
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
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }
    }
}