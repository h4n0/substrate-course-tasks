#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{collections::HashMap, lazy::Lazy};

    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Lazy<Balance>,
        balances: HashMap<AccountId, Balance>,
        allowances: HashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientApproval,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = HashMap::new();
            balances.insert(caller, supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: supply,
            });

            Self {
                total_supply: Lazy::new(supply),
                balances,
                allowances: HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, who: AccountId) -> Balance {
            self.balances.get(&who).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self, who: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(who, spender)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.inner_transfer(from, to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, to), value);

            self.env().emit_event(Approval {
                owner,
                spender: to,
                value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientApproval);
            }

            self.inner_transfer(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);

            Ok(())
        }

        pub fn inner_transfer(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn ctor_works() {
            let total_supply = 100000;
            let erc20 = Erc20::new(total_supply);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            assert_eq!(erc20.total_supply(), total_supply);
        }

        #[ink::test]
        fn transfer_works() {
            let total_supply = 100000;
            let mut erc20 = Erc20::new(total_supply);

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

            assert_eq!(erc20.balance_of(accounts.alice), total_supply);
            assert_eq!(erc20.balance_of(accounts.bob), 0);

            assert_eq!(erc20.transfer(accounts.bob, 500), Ok(()));
            assert_eq!(erc20.balance_of(accounts.bob), 500);
            assert_eq!(erc20.balance_of(accounts.alice), total_supply - 500);
        }

        #[ink::test]
        fn transfer_fails_insufficient_balance() {
            let total_supply = 100;
            let mut erc20 = Erc20::new(total_supply);

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

            assert_eq!(
                erc20.transfer(accounts.bob, 500),
                Err(Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let total_supply = 100000;
            let mut erc20 = Erc20::new(total_supply);

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

            assert_eq!(erc20.approve(accounts.bob, 500), Ok(()));
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 500);

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or_else(|_| [0x0; 32].into());
            // Create call.
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 500),
                Ok(())
            );
            assert_eq!(erc20.balance_of(accounts.eve), 500);
        }

        #[ink::test]
        fn transfer_from_fails_with_insufficient_approval() {
            let total_supply = 100000;
            let mut erc20 = Erc20::new(total_supply);

            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()?;

            assert_eq!(erc20.approve(accounts.bob, 100), Ok(()));
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 100);

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or_else(|_| [0x0; 32].into());
            // Create call.
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.eve, 500),
                Err(Error::InsufficientApproval)
            );
        }
    }
}
