#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
	use ink_storage::{
			collections::HashMap,
			lazy::Lazy,
		};

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
		to: Option<AccountId>,
		value: Balance,
	}

	#[ink(event)]
	pub struct Approval {
		#[ink(topic)]
		owner: AccountId,
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

			Self::env().emit_event(Transfer{
				from: None,
				to: Some(caller),
				value: supply,
			});

			Self{
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

			self.env().emit_event(
				Approval{
					owner,
					spender: to,
					value
				}
			);
			Ok(())
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId,value: Balance) -> Result<()> {
			let caller = self.env().caller();
			let allowance = self.allowance(from, caller);
			if allowance < value {
				return Err(Error::InsufficientApproval);
			}

			self.inner_transfer(from, to, value)?;
			self.allowances.insert((from, caller), allowance - value);

			Ok(())
        }

        pub fn inner_transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
			let from_balance = self.balance_of(from);
			if from_balance < value {
				return Err(Error::InsufficientBalance);
			}
			self.balances.insert(from, from_balance - value);
			let to_balance = self.balance_of(to);
			self.balances.insert(to, to_balance + value);

			self.env().emit_event(
				Transfer{
					from: Some(from),
					to: Some(to),
					value
				}
			);

			Ok(())
        }
	}
}