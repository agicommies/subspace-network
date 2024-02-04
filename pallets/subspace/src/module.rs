use super::*;
use frame_support::pallet_prelude::{Decode, DispatchResult, Encode};

extern crate alloc;
use alloc::vec::Vec;
use sp_arithmetic::per_things::Percent;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct ModuleStats<T: Config> {
	pub last_update: u64,
	pub registration_block: u64,
	pub stake_from: Vec<(T::AccountId, u64)>, /* map of key to stake on this module/key *
	                                           * (includes delegations) */
	pub emission: u64,
	pub incentive: u16,
	pub dividends: u16,
	pub weights: Vec<(u16, u16)>, // Vec of (uid, weight)
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct ModuleInfo<T: Config> {
	params: ModuleParams<T>,
	stats: ModuleStats<T>,
}

impl<T: Config> Pallet<T> {
	pub fn do_update_module(
		origin: T::RuntimeOrigin,
		netuid: u16,
		params: ModuleParams<T>,
	) -> DispatchResult {
		// --- 1. We check the callers (key) signature.
		let key = ensure_signed(origin)?;
		let uid: u16 = Self::get_uid_for_key(netuid, &key);
		Self::check_module_params(&params)?;
		Self::set_module_params(netuid, uid, params);
		// --- 8. Return is successful dispatch.
		Ok(())
	}

	pub fn check_module_params(params: &ModuleParams<T>) -> DispatchResult {
		// if len(name) > 0, then we update the name.
		assert!(params.name.len() > 0);
		ensure!(
			params.name.len() <= Self::get_global_max_name_length() as usize,
			Error::<T>::ModuleNameTooLong
		);
		assert!(params.address.len() > 0);
		ensure!(
			params.address.len() <= Self::get_global_max_name_length() as usize,
			Error::<T>::ModuleAddressTooLong
		);
		// delegation fee is a percent
		Ok(())
	}

	pub fn module_params(netuid: u16, uid: u16) -> ModuleParams<T> {
		ModuleParams {
			name: Self::get_module_name(netuid, uid),
			address: Self::get_module_address(netuid, uid),
			delegation_fee: Self::get_module_delegation_fee(netuid, uid),
			controller: Self::get_key_for_uid(netuid, uid),
		}
	}

	pub fn set_module_params(netuid: u16, uid: u16, module_params: ModuleParams<T>) {
		Self::set_module_name(netuid, uid, module_params.name);
		Self::set_module_address(netuid, uid, module_params.address);
		Self::set_module_delegation_fee(netuid, uid, module_params.delegation_fee);
	}

	pub fn get_module_address(netuid: u16, uid: u16) -> Vec<u8> {
		return Address::<T>::get(netuid, uid)
	}

	pub fn set_module_address(netuid: u16, uid: u16, address: Vec<u8>) {
		Address::<T>::insert(netuid, uid, address);
	}

	pub fn get_module_delegation_fee(netuid: u16, uid: u16) -> Percent {
		let key = Self::get_key_for_uid(netuid, uid);
		let delegation_fee: Percent = DelegationFee::<T>::get(netuid, key);
		delegation_fee
	}

	pub fn set_module_delegation_fee(netuid: u16, uid: u16, delegation_fee: Percent) {
		let key = Self::get_key_for_uid(netuid, uid);
		DelegationFee::<T>::insert(netuid, key, delegation_fee);
	}

	pub fn get_module_name(netuid: u16, uid: u16) -> Vec<u8> {
		return Name::<T>::get(netuid, uid)
	}

	pub fn set_module_name(netuid: u16, uid: u16, name: Vec<u8>) {
		Name::<T>::insert(netuid, uid, name.clone());
	}

	// Replace the module under this uid.
	pub fn remove_module(netuid: u16, uid: u16) {
		// 1. Get the old key under this position.
		let n = Self::get_subnet_n(netuid);
		if n == 0 {
			// No modules in the network.
			return;
		}
		let uid_key: T::AccountId = Keys::<T>::get(netuid, uid);
		let replace_uid = n - 1;
		let replace_key: T::AccountId = Keys::<T>::get(netuid, replace_uid);

		log::debug!(
			"remote_subnet( netuid: {:?} | uid : {:?} | new_key: {:?} ) ",
			netuid,
			uid,
			uid_key
		);

		// HANDLE THE KEY AND UID ASSOCIATIONS
		Uids::<T>::insert(netuid, replace_key.clone(), uid); // Remove old key - uid association.
		Keys::<T>::insert(netuid, uid, replace_key.clone()); // Make key - uid association.
		Uids::<T>::remove(netuid, uid_key.clone()); // Remove old key - uid association.
		Keys::<T>::remove(netuid, replace_uid); // Remove key - uid association.

		// let mut delegation_fee: Percent = DelegationFee::<T>::get(netuid, uid_key.clone());

		// swap consensus vectors
		let mut subnet_state = SubnetStateStorage::<T>::get(netuid);

		subnet_state.incentives[uid as usize] = subnet_state.incentives[replace_uid as usize];
		subnet_state.dividends[uid as usize] = subnet_state.dividends[replace_uid as usize];
		subnet_state.emissions[uid as usize] = subnet_state.emissions[replace_uid as usize];
		subnet_state.last_updates[uid as usize] = subnet_state.last_updates[replace_uid as usize];

		// pop the last element (which is now a duplicate)
		subnet_state.incentives.pop();
		subnet_state.dividends.pop();
		subnet_state.emissions.pop();
		subnet_state.last_updates.pop();

		SubnetStateStorage::<T>::insert(netuid, subnet_state);

		// SWAP WEIGHTS
		Weights::<T>::insert(netuid, uid, Weights::<T>::get(netuid, replace_uid)); // Make uid - key association.
		Weights::<T>::remove(netuid, replace_uid); // Make uid - key association.

		// HANDLE THE REGISTRATION BLOCK
		RegistrationBlock::<T>::insert(
			netuid,
			uid,
			RegistrationBlock::<T>::get(netuid, replace_uid),
		); // Fill block at registration.
		RegistrationBlock::<T>::remove(netuid, replace_uid); // Fill block at registration.

		// HANDLE THE ADDRESS
		Address::<T>::insert(netuid, uid, Address::<T>::get(netuid, replace_uid)); // Fill module info.
		Address::<T>::remove(netuid, replace_uid); // Fill module info.

		// HANDLE THE NAMES
		Name::<T>::insert(netuid, uid, Name::<T>::get(netuid, replace_uid)); // Fill module namespace.
		Name::<T>::remove(netuid, replace_uid); // Fill module namespace.

		// HANDLE THE DELEGATION FEE
		DelegationFee::<T>::insert(
			netuid,
			replace_key.clone(),
			DelegationFee::<T>::get(netuid, uid_key.clone()),
		); // Make uid - key association.
		DelegationFee::<T>::remove(netuid, uid_key.clone()); // Make uid - key association.

		// 3. Remove the network if it is empty.
		let mut subnet_state = SubnetStateStorage::<T>::get(netuid);

		if subnet_state.n > 0 {
			subnet_state.n -= 1;
		}

		SubnetStateStorage::<T>::insert(netuid, subnet_state);

		// remove the network if it is empty
		if Self::get_subnet_n(netuid) == 0 {
			Self::remove_subnet(netuid);
		}

		// remove stake from old key and add to new key
		Self::remove_stake_from_storage(netuid, &uid_key);
	}

	// Appends the uid to the network.
	pub fn append_module(netuid: u16, key: &T::AccountId, name: Vec<u8>, address: Vec<u8>) -> u16 {
		// 1. Get the next uid. This is always equal to subnetwork_n.
		let uid: u16 = Self::get_subnet_n(netuid);
		let block_number = Self::get_current_block_as_u64();
		log::debug!("append_module( netuid: {:?} | uid: {:?} | new_key: {:?} ) ", netuid, key, uid);

		let mut subnet_state = SubnetStateStorage::<T>::get(netuid);

		// TODO: handle errors
		let _ = subnet_state.incentives.try_push(0);
		let _ = subnet_state.dividends.try_push(0);
		let _ = subnet_state.emissions.try_push(0);
		let _ = subnet_state.last_updates.try_push(block_number);

		SubnetStateStorage::<T>::insert(netuid, subnet_state);

		// 4. Insert new account information.
		Keys::<T>::insert(netuid, uid, key.clone()); // Make key - uid association.
		Uids::<T>::insert(netuid, key.clone(), uid); // Make uid - key association.
		RegistrationBlock::<T>::insert(netuid, uid, block_number); // Fill block at registration.
		Name::<T>::insert(netuid, uid, name.clone()); // Fill module namespace.
		Address::<T>::insert(netuid, uid, address.clone()); // Fill module info.
		DelegationFee::<T>::insert(
			netuid,
			key.clone(),
			DelegationFee::<T>::get(netuid, key.clone()),
		); // Make uid - key association.

		let mut subnet_state = SubnetStateStorage::<T>::get(netuid);

		subnet_state.n += 1;

		SubnetStateStorage::<T>::insert(netuid, subnet_state);

		uid
	}

	pub fn get_modules_stats(netuid: u16) -> Vec<ModuleStats<T>> {
		if !Self::if_subnet_exist(netuid) {
			return Vec::new()
		}

		let mut modules = Vec::new();
		let n = Self::get_subnet_n(netuid);
		for uid in 0..n {
			let module = Self::get_module_stats(netuid, uid);
			modules.push(module);
		}

		modules
	}

	pub fn get_module_stats(netuid: u16, uid: u16) -> ModuleStats<T> {
		let emission = Self::get_emission_for_uid(netuid, uid as u16);
		let incentive = Self::get_incentive_for_uid(netuid, uid as u16);
		let dividends = Self::get_dividends_for_uid(netuid, uid as u16);
		let last_update = Self::get_last_update_for_uid(netuid, uid as u16);
		let weights = <Weights<T>>::get(netuid, uid)
			.iter()
			.filter_map(|(i, w)| if *w > 0 { Some(((*i).into(), (*w).into())) } else { None })
			.collect::<Vec<(u16, u16)>>();
		let stake_from: Vec<(T::AccountId, u64)> = Stake::<T>::iter_prefix(netuid)
			.map(|(key, stake)| (key, stake.into()))
			.collect();
		let registration_block = Self::get_registration_block_for_uid(netuid, uid as u16);

		ModuleStats {
			stake_from,
			emission: emission.into(),
			incentive: incentive.into(),
			dividends: dividends.into(),
			last_update: last_update.into(),
			registration_block: registration_block.into(),
			weights,
		}
	}
}
