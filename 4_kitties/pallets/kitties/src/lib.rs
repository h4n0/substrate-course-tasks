#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

use codec::{Codec, Decode, Encode};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency}};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, Saturating, StaticLookup};

	type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode)]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub list_price: Option<BalanceOf<T>>,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter
			+ MaybeSerializeDeserialize
			+ Bounded
			+ AtLeast32BitUnsigned
			+ Copy
			+ MaxEncodedLen;
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type ReserveAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

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
		KittyAlreadyOwned,
KittyWithoutOwner,
KittyNotForSale,
InsufficientReserveBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&who);

			Self::kitty_create(who, kitty_id, dna)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: <T::Lookup as StaticLookup>::Source,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			Self::kitty_transfer(who.clone(), recipient.clone(), kitty_id)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: T::KittyIndex,
			kitty_id_2: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = Self::next_kitty_id()?;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..new_dna.len() {
				new_dna[i] = (selector[i] & kitty_1.dna[i]) | (!selector[i] & kitty_2.dna[i]);
			}

			Self::kitty_create(who, kitty_id, new_dna)?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			list_price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			// Ensure kitty_id is valid
			let _ = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			// Update the list price
			Kitties::<T>::mutate(kitty_id, |kitty|{
				if let Some(k) =  kitty {
					k.list_price = Some(list_price);
				}
			});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn buy(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get the original kitty owner
			let original_owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::KittyWithoutOwner)?;

			// Make sure the original owner is not the buyer
			ensure!(who != original_owner, Error::<T>::KittyAlreadyOwned);

			// Ensure kitty_id is valid
			let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			// Get the kitty list price only if it's for sale
			let kitty_price = kitty.list_price.ok_or(Error::<T>::KittyNotForSale)?;

			T::Currency::transfer(&who, &original_owner, kitty_price, ExistenceRequirement::AllowDeath)?;

			Self::kitty_transfer(original_owner, who, kitty_id)?;

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

		// Calculate the next kitty index
		fn next_kitty_id() -> Result<T::KittyIndex, Error<T>> {
			match Self::kitties_count() {
				Some(id) => {
					ensure!(id <= T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					Ok(id)
				}
				None => Ok(T::KittyIndex::min_value().saturating_add(T::KittyIndex::from(1u8)))
			}
		}

		// Create kitty
		fn kitty_create(who: T::AccountId, kitty_id: T::KittyIndex, dna:[u8; 16]) -> Result<(), Error<T>> {

			// Reserve amount of token
			T::Currency::reserve(&who, T::ReserveAmount::get()).map_err(|_| Error::<T>::InsufficientReserveBalance)?;

			Kitties::<T>::insert(kitty_id, Some(Kitty{dna: dna, list_price: None}));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id.saturating_add(T::KittyIndex::from(1u8)));

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		// Transfer kitty ownership and reserved tokens
		fn kitty_transfer(
			sender: T::AccountId,
			recipient: T::AccountId,
			kitty_id: T::KittyIndex
		) -> Result<(), Error<T>> {

			// Ensure the given kitty belongs to the sender
			ensure!(Some(sender.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			// Reserve amount of token on recipient
			T::Currency::reserve(&recipient, T::ReserveAmount::get()).map_err(|_| Error::<T>::InsufficientReserveBalance)?;

			Owner::<T>::insert(kitty_id, Some(recipient.clone()));

			// Unreserve balance of sender
			T::Currency::unreserve(&sender, T::ReserveAmount::get());

			Self::deposit_event(Event::KittyTransferred(sender, recipient, kitty_id));

			Ok(())
		}

	}
}
