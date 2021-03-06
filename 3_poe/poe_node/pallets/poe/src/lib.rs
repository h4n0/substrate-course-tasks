#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// This module is for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    pallet_prelude::*
  };
  use frame_system::pallet_prelude::*;
  use sp_std::vec::Vec;

  #[pallet::config]
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	type ClaimLengthLimit: Get<usize>;

  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn proofs)]
  pub type Proofs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,
    (T::AccountId, T::BlockNumber)
  >;

  #[pallet::event]
  #[pallet::metadata(T::AccountId = "AccountId")]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    ClaimCreated(T::AccountId, Vec<u8>),
    ClaimRevoked(T::AccountId, Vec<u8>),
    ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
  }

  #[pallet::error]
  pub enum Error<T> {
    ProofAlreadyExist,
    ClaimNotExist,
    NotClaimOwner,
	ClaimLengthLimitExceed
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::weight(0)]
    pub fn create_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>
    ) -> DispatchResultWithPostInfo {

      let sender = ensure_signed(origin)?;

      ensure!(claim.len() <= T::ClaimLengthLimit::get(), Error::<T>::ClaimLengthLimitExceed);

      ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

      Proofs::<T>::insert(&claim, (sender.clone(), frame_system::Pallet::<T>::block_number()));

      Self::deposit_event(Event::ClaimCreated(sender, claim));
      Ok(().into())
    }

    #[pallet::weight(0)]
    pub fn revoke_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>
    ) -> DispatchResultWithPostInfo {

      let sender = ensure_signed(origin)?;

      let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

      ensure!(owner == sender, Error::<T>::NotClaimOwner);

      Proofs::<T>::remove(&claim);

      Self::deposit_event(Event::ClaimRevoked(sender, claim));
      Ok(().into())
    }

    #[pallet::weight(0)]
    pub fn transfer_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>,
	  recipient: T::AccountId
    ) -> DispatchResultWithPostInfo {

      let sender = ensure_signed(origin)?;

	  // Ensure the claim is existing
      let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

	  // Ensure the transaction sender is the claim owner
      ensure!(owner == sender, Error::<T>::NotClaimOwner);

	  // Get the current block number
	  let cur_block = frame_system::Pallet::<T>::block_number();

	  // Update the storage value under this claim
      Proofs::<T>::mutate(&claim, | value | {
		  match value.as_mut() {
		   Some(v) => {
			   v.0 = recipient.clone();
			   v.1 = cur_block;
		   },
		   None => {}
		  }
	  });

      Self::deposit_event(Event::ClaimTransferred(sender, recipient, claim));
      Ok(().into())
    }

  }

}
