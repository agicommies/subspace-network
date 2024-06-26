use super::*;

use frame_support::{
    pallet_prelude::DispatchResult, storage::IterableStorageMap, IterableStorageDoubleMap,
};

use self::global::BurnConfiguration;
use sp_runtime::{BoundedVec, DispatchError};
use sp_std::vec::Vec;
use substrate_fixed::types::I64F64;

// ---------------------------------
// Subnet Parameters
// ---------------------------------

#[derive(Debug)]
pub struct SubnetChangeset<T: Config> {
    params: SubnetParams<T>,
}

impl<T: Config> SubnetChangeset<T> {
    pub fn new(params: SubnetParams<T>) -> Result<Self, DispatchError> {
        Self::validate_params(None, &params)?;
        Ok(Self { params })
    }

    pub fn update(netuid: u16, params: SubnetParams<T>) -> Result<Self, DispatchError> {
        Self::validate_params(Some(netuid), &params)?;
        Ok(Self { params })
    }

    pub fn apply(self, netuid: u16) -> DispatchResult {
        Self::validate_params(Some(netuid), &self.params)?;

        Pallet::<T>::set_max_allowed_uids(netuid, self.params.max_allowed_uids)?;

        SubnetNames::<T>::insert(netuid, self.params.name.into_inner());
        Founder::<T>::insert(netuid, &self.params.founder);
        FounderShare::<T>::insert(netuid, self.params.founder_share);
        Tempo::<T>::insert(netuid, self.params.tempo);
        ImmunityPeriod::<T>::insert(netuid, self.params.immunity_period);
        MaxAllowedWeights::<T>::insert(netuid, self.params.max_allowed_weights);
        MaxWeightAge::<T>::insert(netuid, self.params.max_weight_age);
        MinAllowedWeights::<T>::insert(netuid, self.params.min_allowed_weights);
        MinStake::<T>::insert(netuid, self.params.min_stake);
        TrustRatio::<T>::insert(netuid, self.params.trust_ratio);
        IncentiveRatio::<T>::insert(netuid, self.params.incentive_ratio);
        BondsMovingAverage::<T>::insert(netuid, self.params.bonds_ma);
        TargetRegistrationsInterval::<T>::insert(netuid, self.params.target_registrations_interval);
        TargetRegistrationsPerInterval::<T>::insert(
            netuid,
            self.params.target_registrations_per_interval,
        );
        MaxRegistrationsPerInterval::<T>::insert(
            netuid,
            self.params.max_registrations_per_interval,
        );

        AdjustmentAlpha::<T>::insert(netuid, self.params.adjustment_alpha);
        if self.params.maximum_set_weight_calls_per_epoch == 0 {
            MaximumSetWeightCallsPerEpoch::<T>::remove(netuid);
        } else {
            MaximumSetWeightCallsPerEpoch::<T>::insert(
                netuid,
                self.params.maximum_set_weight_calls_per_epoch,
            );
        }

        T::update_subnet_governance_configuration(netuid, self.params.governance_config)?;

        Pallet::<T>::deposit_event(Event::SubnetParamsUpdated(netuid));

        Ok(())
    }

    pub fn validate_params(netuid: Option<u16>, params: &SubnetParams<T>) -> DispatchResult {
        // checks if params are valid
        let global_params = Pallet::<T>::global_params();

        // check valid tempo
        ensure!(
            params.min_allowed_weights <= params.max_allowed_weights,
            Error::<T>::InvalidMinAllowedWeights
        );

        ensure!(
            params.max_allowed_weights <= global_params.max_allowed_weights,
            Error::<T>::InvalidMaxAllowedWeights
        );

        ensure!(
            params.min_allowed_weights >= 1,
            Error::<T>::InvalidMinAllowedWeights
        );

        // lower tempos might significantly slow down the chain
        ensure!(params.tempo >= 25, Error::<T>::InvalidTempo);

        ensure!(
            params.max_weight_age > params.tempo as u64,
            Error::<T>::InvalidMaxWeightAge
        );

        // ensure the trust_ratio is between 0 and 100
        ensure!(params.trust_ratio <= 100, Error::<T>::InvalidTrustRatio);

        ensure!(
            params.max_allowed_uids > 0,
            Error::<T>::InvalidMaxAllowedUids
        );

        ensure!(params.founder_share <= 100, Error::<T>::InvalidFounderShare);

        ensure!(
            params.founder_share >= FloorFounderShare::<T>::get() as u16,
            Error::<T>::InvalidFounderShare
        );

        ensure!(
            params.incentive_ratio <= 100,
            Error::<T>::InvalidIncentiveRatio
        );

        ensure!(
            params.max_allowed_weights <= MaxAllowedWeightsGlobal::<T>::get(),
            Error::<T>::InvalidMaxAllowedWeights
        );

        // match registration parameters
        ensure!(
            params.target_registrations_interval >= 10,
            Error::<T>::InvalidTargetRegistrationsInterval
        );

        ensure!(
            params.target_registrations_per_interval >= 1,
            Error::<T>::InvalidTargetRegistrationsPerInterval
        );

        ensure!(
            params.max_registrations_per_interval >= 1,
            Error::<T>::InvalidMaxRegistrationsPerInterval
        );

        ensure!(
            params.adjustment_alpha > 0,
            Error::<T>::InvalidAdjustmentAlpha
        );

        match Pallet::<T>::get_netuid_for_name(&params.name) {
            Some(id) if netuid.is_some_and(|netuid| netuid == id) => { /* subnet kept same name */ }
            Some(_) => return Err(Error::<T>::SubnetNameAlreadyExists.into()),
            None => {
                let name = &params.name;
                let min = MinNameLength::<T>::get() as usize;
                let max = MaxNameLength::<T>::get() as usize;
                ensure!(!name.is_empty(), Error::<T>::InvalidSubnetName);
                ensure!(name.len() >= min, Error::<T>::SubnetNameTooShort);
                ensure!(name.len() <= max, Error::<T>::SubnetNameTooLong);
                core::str::from_utf8(name).map_err(|_| Error::<T>::InvalidSubnetName)?;
            }
        }

        Ok(())
    }
}

impl<T: Config> Pallet<T> {
    pub fn subnet_params(netuid: u16) -> SubnetParams<T> {
        SubnetParams {
            founder: Founder::<T>::get(netuid),
            founder_share: FounderShare::<T>::get(netuid),
            tempo: Tempo::<T>::get(netuid),
            immunity_period: ImmunityPeriod::<T>::get(netuid),
            max_allowed_weights: MaxAllowedWeights::<T>::get(netuid),
            max_allowed_uids: MaxAllowedUids::<T>::get(netuid),
            max_weight_age: MaxWeightAge::<T>::get(netuid),
            min_allowed_weights: MinAllowedWeights::<T>::get(netuid),
            min_stake: MinStake::<T>::get(netuid),
            name: BoundedVec::truncate_from(SubnetNames::<T>::get(netuid)),
            trust_ratio: TrustRatio::<T>::get(netuid),
            incentive_ratio: IncentiveRatio::<T>::get(netuid),
            maximum_set_weight_calls_per_epoch: MaximumSetWeightCallsPerEpoch::<T>::get(netuid)
                .unwrap_or_default(),
            bonds_ma: BondsMovingAverage::<T>::get(netuid),
            target_registrations_interval: TargetRegistrationsInterval::<T>::get(netuid),
            target_registrations_per_interval: TargetRegistrationsPerInterval::<T>::get(netuid),
            max_registrations_per_interval: MaxRegistrationsPerInterval::<T>::get(netuid),
            adjustment_alpha: AdjustmentAlpha::<T>::get(netuid),
            governance_config: T::get_subnet_governance_configuration(netuid),
        }
    }

    // ---------------------------------
    // Adding Subnets
    // ---------------------------------

    pub fn add_subnet(
        changeset: SubnetChangeset<T>,
        netuid: Option<u16>,
    ) -> Result<u16, DispatchError> {
        let netuid = netuid.unwrap_or_else(|| match SubnetGaps::<T>::get().first().copied() {
            Some(removed) => removed,
            None => TotalSubnets::<T>::get(),
        });

        let name = changeset.params.name.clone();
        changeset.apply(netuid)?;
        TotalSubnets::<T>::mutate(|n| *n = n.saturating_add(1));
        N::<T>::insert(netuid, 0);
        T::set_subnet_emission_storage(netuid, 0);
        SubnetRegistrationsThisInterval::<T>::mutate(|value| *value = value.saturating_add(1));

        // Insert the minimum burn to the netuid,
        // to prevent free registrations the first target registration interval.
        let BurnConfiguration { min_burn, .. } = BurnConfig::<T>::get();
        Burn::<T>::insert(netuid, min_burn);

        SubnetGaps::<T>::mutate(|subnets| subnets.remove(&netuid));

        // --- 6. Emit the new network event.
        Self::deposit_event(Event::NetworkAdded(netuid, name.into_inner()));

        Ok(netuid)
    }

    // Removing subnets
    // ---------------------------------
    // TODO: improve safety, to check if all storages are,
    // actually being deleted, from subnet_params struct,
    // and other storage items, in consensus vectors etc..
    pub fn remove_subnet(netuid: u16) -> u16 {
        // --- 0. Ensure the network to be removed exists.
        if !Self::if_subnet_exist(netuid) {
            return 0;
        }

        // TODO:
        // If the key is only present on the subnet, it is getting deregistered from.
        // Then we want to unstake, otherwise we don't do that.
        // Decide how to proceed with this
        // Self::remove_netuid_stake_storage(netuid);

        // --- 1. Erase all subnet module data.
        // ====================================

        SubnetNames::<T>::remove(netuid);
        let _ = Name::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Address::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Metadata::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = DelegationFee::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 2. Remove consnesus vectors
        // ===============================

        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);
        Active::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(netuid);
        LastUpdate::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        ValidatorPermits::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);
        let _ = RegistrationBlock::<T>::clear_prefix(netuid, u32::MAX, None);
        T::remove_subnet_emission_storage(netuid);

        // --- 3. Erase subnet parameters.
        // ===============================

        Founder::<T>::remove(netuid);
        FounderShare::<T>::remove(netuid);
        Tempo::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        MaxAllowedWeights::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        MaxWeightAge::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        MinStake::<T>::remove(netuid);
        TrustRatio::<T>::remove(netuid);
        IncentiveRatio::<T>::remove(netuid);
        MaximumSetWeightCallsPerEpoch::<T>::remove(netuid);
        BondsMovingAverage::<T>::remove(netuid);
        TargetRegistrationsInterval::<T>::remove(netuid);
        TargetRegistrationsPerInterval::<T>::remove(netuid);
        MaxRegistrationsPerInterval::<T>::remove(netuid);
        AdjustmentAlpha::<T>::remove(netuid);

        T::handle_subnet_removal(netuid);

        // --- 4 Adjust the total number of subnets. and remove the subnet from the list of subnets.
        // =========================================================================================

        N::<T>::remove(netuid);
        TotalSubnets::<T>::mutate(|val| *val = val.saturating_sub(1));
        SubnetGaps::<T>::mutate(|subnets| subnets.insert(netuid));

        // --- 5. Emit the event.
        // ======================

        Self::deposit_event(Event::NetworkRemoved(netuid));

        netuid
    }
    // ---------------------------------
    // Updating Subnets
    // ---------------------------------

    pub fn do_update_subnet(
        origin: T::RuntimeOrigin,
        netuid: u16,
        changeset: SubnetChangeset<T>,
    ) -> DispatchResult {
        let key = ensure_signed(origin)?;

        // -- 1. Make sure the netuid exists
        ensure!(
            SubnetNames::<T>::contains_key(netuid),
            Error::<T>::NetuidDoesNotExist
        );

        // --2. Ensury Authority - only the founder can update the network on authority mode.
        ensure!(Founder::<T>::get(netuid) == key, Error::<T>::NotFounder);

        // -4. Apply the changeset.
        changeset.apply(netuid)?;

        // --- 5. Ok and done.
        Ok(())
    }

    // ---------------------------------
    // Setters
    // ---------------------------------

    fn set_max_allowed_uids(netuid: u16, mut max_allowed_uids: u16) -> DispatchResult {
        let n: u16 = N::<T>::get(netuid);
        if max_allowed_uids < n {
            // limit it at 256 at a time
            let mut remainder_n: u16 = n.saturating_sub(max_allowed_uids);
            let max_remainder = 256;
            if remainder_n > max_remainder {
                // remove the modules in small amounts, as this can be a heavy load on the chain
                remainder_n = max_remainder;
                max_allowed_uids = n.saturating_sub(remainder_n);
            }
            // remove the modules by adding the to the deregister queue
            for i in 0..remainder_n {
                let next_uid = n.saturating_sub(1).saturating_sub(i);
                Self::remove_module(netuid, next_uid, true)?;
            }
        }

        MaxAllowedUids::<T>::insert(netuid, max_allowed_uids);

        Ok(())
    }

    pub fn set_last_update_for_uid(netuid: u16, uid: u16, last_update: u64) {
        LastUpdate::<T>::mutate(netuid, |vec| {
            if let Some(idx) = vec.get_mut(uid as usize) {
                *idx = last_update;
            }
        });
    }

    // ---------------------------------
    // Getters
    // ---------------------------------

    pub fn get_min_allowed_weights(netuid: u16) -> u16 {
        let min_allowed_weights = MinAllowedWeights::<T>::get(netuid);
        min_allowed_weights.min(N::<T>::get(netuid))
    }

    pub fn get_uids(netuid: u16) -> Vec<u16> {
        (0..N::<T>::get(netuid)).collect()
    }

    pub fn get_keys(netuid: u16) -> Vec<T::AccountId> {
        Self::get_uids(netuid)
            .into_iter()
            .map(|uid| Self::get_key_for_uid(netuid, uid).unwrap())
            .collect()
    }

    pub fn get_uid_key_tuples(netuid: u16) -> Vec<(u16, T::AccountId)> {
        (0..N::<T>::get(netuid))
            .map(|uid| (uid, Self::get_key_for_uid(netuid, uid).unwrap()))
            .collect()
    }

    pub fn get_names(netuid: u16) -> Vec<Vec<u8>> {
        <Name<T> as IterableStorageDoubleMap<u16, u16, Vec<u8>>>::iter_prefix(netuid)
            .map(|(_, name)| name)
            .collect()
    }

    pub fn get_addresses(netuid: u16) -> Vec<T::AccountId> {
        <Uids<T> as IterableStorageDoubleMap<u16, T::AccountId, u16>>::iter_prefix(netuid)
            .map(|(key, _)| key)
            .collect()
    }

    pub fn get_netuid_for_name(name: &[u8]) -> Option<u16> {
        SubnetNames::<T>::iter().find(|(_, n)| n == name).map(|(id, _)| id)
    }
    // Returs the key under the network uid as a Result. Ok if the uid is taken.
    pub fn get_key_for_uid(netuid: u16, module_uid: u16) -> Option<T::AccountId> {
        Keys::<T>::try_get(netuid, module_uid).ok()
    }
    // Returns the uid of the key in the network as a Result. Ok if the key has a slot.
    pub fn get_uid_for_key(netuid: u16, key: &T::AccountId) -> u16 {
        Uids::<T>::get(netuid, key).unwrap_or(0)
    }

    pub fn get_current_block_number() -> u64 {
        TryInto::try_into(<frame_system::Pallet<T>>::block_number())
            .ok()
            .expect("blockchain will not exceed 2^64 blocks; QED.")
    }

    pub fn get_emission_for_uid(netuid: u16, uid: u16) -> u64 {
        Emission::<T>::get(netuid).get(uid as usize).copied().unwrap_or_default()
    }

    pub fn get_incentive_for_uid(netuid: u16, uid: u16) -> u16 {
        Incentive::<T>::get(netuid).get(uid as usize).copied().unwrap_or_default()
    }

    pub fn get_dividends_for_uid(netuid: u16, uid: u16) -> u16 {
        Dividends::<T>::get(netuid).get(uid as usize).copied().unwrap_or_default()
    }

    pub fn get_last_update_for_uid(netuid: u16, uid: u16) -> u64 {
        LastUpdate::<T>::get(netuid).get(uid as usize).copied().unwrap_or_default()
    }

    // ---------------------------------
    // Utility
    // ---------------------------------
    pub fn calculate_founder_emission(netuid: u16, mut token_emission: u64) -> (u64, u64) {
        let founder_share: u16 = FounderShare::<T>::get(netuid).min(100);
        if founder_share == 0u16 {
            return (token_emission, 0);
        }

        let founder_emission_ratio: I64F64 = I64F64::from_num(founder_share.min(100))
            .checked_div(I64F64::from_num(100))
            .unwrap_or(I64F64::from_num(0));

        let founder_emission = founder_emission_ratio
            .checked_mul(I64F64::from_num(token_emission))
            .map(|result| result.to_num::<u64>())
            .unwrap_or(0);

        token_emission = token_emission.saturating_sub(founder_emission);

        (token_emission, founder_emission)
    }

    pub fn get_ownership_ratios(
        netuid: u16,
        module_key: &T::AccountId,
    ) -> Vec<(T::AccountId, I64F64)> {
        let stake_from_vector = Self::get_stake_from_vector(module_key);
        let _uid = Self::get_uid_for_key(netuid, module_key);
        let mut total_stake_from: I64F64 = I64F64::from_num(0);

        let mut ownership_vector: Vec<(T::AccountId, I64F64)> = Vec::new();

        for (k, v) in stake_from_vector.clone().into_iter() {
            let ownership = I64F64::from_num(v);
            ownership_vector.push((k.clone(), ownership));
            total_stake_from = total_stake_from.saturating_add(ownership);
        }

        // add the module itself, if it has stake of its own
        if total_stake_from == I64F64::from_num(0) {
            ownership_vector.push((module_key.clone(), I64F64::from_num(0)));
        } else {
            ownership_vector = ownership_vector
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.checked_div(total_stake_from).unwrap_or(I64F64::from_num(0)),
                    )
                })
                .collect();
        }

        ownership_vector
    }

    pub fn is_key_registered_on_any_network(key: &T::AccountId) -> bool {
        Self::netuids().iter().any(|&netuid| Uids::<T>::contains_key(netuid, key))
    }

    pub fn is_registered(network: Option<u16>, key: &T::AccountId) -> bool {
        match network {
            Some(netuid) => Uids::<T>::contains_key(netuid, key),
            None => N::<T>::iter_keys().any(|netuid| Uids::<T>::contains_key(netuid, key)),
        }
    }

    pub fn if_subnet_exist(netuid: u16) -> bool {
        N::<T>::contains_key(netuid)
    }

    pub fn key_registered(netuid: u16, key: &T::AccountId) -> bool {
        Uids::<T>::contains_key(netuid, key)
            || Keys::<T>::iter_prefix_values(netuid).any(|k| &k == key)
    }

    pub fn netuids() -> Vec<u16> {
        <N<T> as IterableStorageMap<u16, u16>>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }
}
