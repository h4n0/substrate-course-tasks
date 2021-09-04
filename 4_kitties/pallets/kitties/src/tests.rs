use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_kitty_succeeds() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		assert_ok!(Kitties::create(Origin::signed(1)));
		assert_eq!(Balances::reserved_balance(1), ReserveAmount::get());
		// kitty_index starts from 1
		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));
		assert_eq!(Kitties::kitties_count(), Some(1));

		System::assert_last_event(Event::Kitties(crate::Event::KittyCreated(1, 1)));
	});
}

#[test]
fn create_kitty_fails_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 5, 0);
		assert_noop!(Kitties::create(Origin::signed(1)), Error::<Test>::InsufficientReserveBalance);
	});
}

#[test]
fn create_kitty_fails_with_count_overflow() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		Kitties::set_kitty_count(KittyIndex::max_value());
		assert_noop!(Kitties::create(Origin::signed(1)), Error::<Test>::KittiesCountOverflow);
	});
}

#[test]
fn transfer_kitty_succeeds() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::transfer(Origin::signed(1), 8, 1));
		assert_eq!(Kitties::owner(1), Some(8));

		assert_eq!(Balances::reserved_balance(1), 0);
		assert_eq!(Balances::reserved_balance(8), ReserveAmount::get());

		System::assert_last_event(Event::Kitties(crate::Event::KittyTransferred(1, 8, 1)));
	});
}

#[test]
fn transfer_kitty_fails_with_not_owner() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_noop!(Kitties::transfer(Origin::signed(5), 8, 1), Error::<Test>::NotOwner);
	});
}

#[test]
fn transfer_kitty_fails_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 5, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_noop!(
			Kitties::transfer(Origin::signed(1), 8, 1),
			Error::<Test>::InsufficientReserveBalance
		);
		assert_eq!(Kitties::owner(1), Some(1));
	});
}

#[test]
fn breed_kitty_succeeds() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(2).is_some(), true);
		assert_eq!(Kitties::owner(2), Some(1));

		assert_ok!(Kitties::breed(Origin::signed(1), 1, 2));
		assert_eq!(Balances::reserved_balance(1), 3 * ReserveAmount::get());

		System::assert_last_event(Event::Kitties(crate::Event::KittyCreated(1, 3)));
	});
}

#[test]
fn breed_kitty_fails_with_invalid_kitty_index() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(2).is_some(), true);
		assert_eq!(Kitties::owner(2), Some(1));

		assert_noop!(Kitties::breed(Origin::signed(1), 1, 3), Error::<Test>::InvalidKittyIndex);
	});
}

#[test]
fn breed_kitty_fails_with_same_parent_error() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_noop!(Kitties::breed(Origin::signed(1), 1, 1), Error::<Test>::SameParentIndex);
	});
}

#[test]
fn sell_kitty_succeeds() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		System::assert_last_event(Event::Kitties(crate::Event::KittyOnSale(1, 1, 50)));
	});
}

#[test]
fn sell_kitty_fails_with_not_owner() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_noop!(Kitties::sell(Origin::signed(2), 1, 50), Error::<Test>::NotOwner);
	});
}

#[test]
fn sell_kitty_fails_with_invalid_index() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_noop!(Kitties::sell(Origin::signed(1), 2, 50), Error::<Test>::InvalidKittyIndex);
	});
}

#[test]
fn buy_kitty_succeeds() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));
		assert_eq!(Balances::reserved_balance(1), ReserveAmount::get());

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		assert_ok!(Kitties::buy(Origin::signed(8), 1));
		assert_eq!(Balances::reserved_balance(1), 0);
		assert_eq!(Balances::free_balance(1), 150);
		assert_eq!(Balances::reserved_balance(8), ReserveAmount::get());
		assert_eq!(Balances::free_balance(8), 50 - ReserveAmount::get());

		System::assert_last_event(Event::Kitties(crate::Event::KittyTransferred(1, 8, 1)));
	});
}

#[test]
fn buy_kitty_fails_with_kitty_no_owner() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		Kitties::set_kitty_owner(1, None);

		assert_noop!(Kitties::buy(Origin::signed(8), 1), Error::<Test>::KittyWithoutOwner);
	});
}

#[test]
fn buy_kitty_fails_with_kitty_already_owned() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		assert_noop!(Kitties::buy(Origin::signed(1), 1), Error::<Test>::KittyAlreadyOwned);
	});
}

#[test]
fn buy_kitty_fails_with_kitty_invalid_index() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		assert_noop!(Kitties::buy(Origin::signed(8), 2), Error::<Test>::InvalidKittyIndex);
	});
}

#[test]
fn buy_kitty_fails_with_kitty_not_for_sale() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 100, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_noop!(Kitties::buy(Origin::signed(8), 1), Error::<Test>::KittyNotForSale);
	});
}

#[test]
fn buy_kitty_fails_with_payment_failing() {
	new_test_ext().execute_with(|| {
		let _ = Balances::set_balance(Origin::root(), 1, 100, 0);
		let _ = Balances::set_balance(Origin::root(), 8, 55, 0);

		assert_ok!(Kitties::create(Origin::signed(1)));

		assert_eq!(Kitties::kitties(1).is_some(), true);
		assert_eq!(Kitties::kitties(1).unwrap().list_price, None);
		assert_eq!(Kitties::owner(1), Some(1));

		assert_ok!(Kitties::sell(Origin::signed(1), 1, 50));
		assert_eq!(Kitties::kitties(1).unwrap().list_price, Some(50));

		assert_noop!(Kitties::buy(Origin::signed(8), 1), Error::<Test>::InsufficientPaymentBalance);
	});
}
