use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_succeeds() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), Some((1, frame_system::Pallet::<Test>::block_number())));
	})
}

#[test]
fn create_claim_fails_with_proof_existing() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ProofAlreadyExist);
	})
}

#[test]
fn create_claim_fails_with_limit_exceeding() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1, 2];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimLengthLimitExceed);
	})
}

#[test]
fn revoke_claim_succeeds() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
		assert_eq!(Proofs::<Test>::get(&claim), None);
	})
}

#[test]
fn revoke_claim_fails_with_claim_not_existing() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_noop!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimNotExist);
	})
}

#[test]
fn transfer_claim_succeeds() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
		assert_eq!(Proofs::<Test>::get(&claim), Some((2, frame_system::Pallet::<Test>::block_number())));
	})
}

#[test]
fn transfer_claim_fails_with_claim_not_existing() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];

		assert_noop!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2), Error::<Test>::ClaimNotExist);
	})
}

#[test]
fn transfer_claim_fails_with_wrong_claim_owner() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(6), claim.clone());

		assert_noop!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2), Error::<Test>::NotClaimOwner);
	})
}
