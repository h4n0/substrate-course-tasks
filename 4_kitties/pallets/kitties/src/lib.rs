#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use codec::{Codec, Decode, Encode};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency}};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, Saturating};

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let kitty_id = Self::next_kitty_id()?;

			let dna = Self::random_value(&who);

			Kitties::<T>::insert(kitty_id, Some(Kitty{dna: dna, list_price: None}));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id.saturating_add(T::KittyIndex::from(1u8)));

			// Reserve amount of token
			let _ = T::Currency::reserve(&who, T::ReserveAmount::get());

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

			Owner::<T>::insert(kitty_id, Some(recipient.clone()));

			Self::deposit_event(Event::KittyTransferred(who, recipient, kitty_id));

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

			Kitties::<T>::insert(kitty_id, Some(Kitty{dna: new_dna, list_price: None}));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_id.saturating_add(T::KittyIndex::from(1u8)));

			Self::deposit_event(Event::KittyCreated(who, kitty_id));

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
				match kitty {
					Some(k) => {
						k.list_price = Some(list_price);
					},
					None => {
					}
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

			let original_owner = Owner::<T>::get(kitty_id).ok_or(Error::<T>::KittyWithoutOwner)?;

			ensure!(who != original_owner, Error::<T>::KittyAlreadyOwned);

			// Ensure kitty_id is valid
			let kitty = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_price = kitty.list_price.ok_or(Error::<T>::KittyNotForSale)?;

			let _ = T::Currency::transfer(&who, &original_owner, kitty_price, ExistenceRequirement::AllowDeath);

			//TODO transfer kitty with a common method

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
				}
				None => Ok(T::KittyIndex::min_value()),
			}
		}
	}
}
