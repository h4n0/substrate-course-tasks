#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use codec::{Decode, Encode};
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, Saturating};

	#[derive(Encode, Decode)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter + MaybeSerializeDeserialize + Bounded + AtLeast32BitUnsigned + Copy + MaxEncodedLen;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id.saturating_add(T::KittyIndex::from(1u8)));

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, recipient: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			Owner::<T>::insert(kitty_id, Some(recipient.clone()));

			Self::deposit_event(Event::KittyTransferred(who, recipient, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna_1 = kitty_1.0;
			let dna_2 = kitty_2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..new_dna.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id.saturating_add(T::KittyIndex::from(1u8)));

			Self::deposit_event(Event::KittyCreated(who, kitty_id));


			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			payload.using_encoded(blake2_128)
		}

		fn next_kitty_id() -> Result<T::KittyIndex, Error<T>> {
			match Self::kitties_count() {
				Some(id) => {
					ensure!(id <= T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					Ok(id)
				},
				None => {
					Ok(T::KittyIndex::min_value())
				}
			}
		}
	}
}
