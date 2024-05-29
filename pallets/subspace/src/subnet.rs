use super::*;

use frame_support::{
    pallet_prelude::DispatchResult, storage::IterableStorageMap, IterableStorageDoubleMap,
};

use self::{global::BurnConfiguration, voting::VoteMode};
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

    pub fn apply(self, netuid: u16) -> Result<(), sp_runtime::DispatchError> {
        Self::validate_params(Some(netuid), &self.params)?;

        SubnetNames::<T>::insert(netuid, self.params.name.into_inner());
        Founder::<T>::insert(netuid, &self.params.founder);
        FounderShare::<T>::insert(netuid, self.params.founder_share);
        Tempo::<T>::insert(netuid, self.params.tempo);
        ImmunityPeriod::<T>::insert(netuid, self.params.immunity_period);
        MaxAllowedWeights::<T>::insert(netuid, self.params.max_allowed_weights);
        Pallet::<T>::set_max_allowed_uids(netuid, self.params.max_allowed_uids);
        MaxWeightAge::<T>::insert(netuid, self.params.max_weight_age);
        MinAllowedWeights::<T>::insert(netuid, self.params.min_allowed_weights);
        MinStake::<T>::insert(netuid, self.params.min_stake);
        TrustRatio::<T>::insert(netuid, self.params.trust_ratio);
        IncentiveRatio::<T>::insert(netuid, self.params.incentive_ratio);
        VoteModeSubnet::<T>::insert(netuid, self.params.vote_mode);

        if self.params.maximum_set_weight_calls_per_epoch == 0 {
            MaximumSetWeightCallsPerEpoch::<T>::remove(netuid);
        } else {
            MaximumSetWeightCallsPerEpoch::<T>::insert(
                netuid,
                self.params.maximum_set_weight_calls_per_epoch,
            );
        }

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
            params.immunity_period > 0,
            Error::<T>::InvalidImmunityPeriod
        );

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
            maximum_set_weight_calls_per_epoch: MaximumSetWeightCallsPerEpoch::<T>::get(netuid),
            vote_mode: VoteModeSubnet::<T>::get(netuid),
            bonds_ma: BondsMovingAverage::<T>::get(netuid),
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
        TotalSubnets::<T>::mutate(|n| *n += 1);
        N::<T>::insert(netuid, 0);
        SubnetEmission::<T>::insert(netuid, 0);

        // Insert the minimum burn to the netuid,
        // to prevent free registrations the first target registration interval.
        let BurnConfiguration { min_burn, .. } = BurnConfig::<T>::get();
        Burn::<T>::insert(netuid, min_burn);

        SubnetGaps::<T>::mutate(|subnets| subnets.remove(&netuid));

        // --- 6. Emit the new network event.
        Self::deposit_event(Event::NetworkAdded(netuid, name.into_inner()));

        Ok(netuid)
    }

    // ---------------------------------
    // Removing subnets
    // ---------------------------------

    pub fn remove_subnet(netuid: u16) -> u16 {
        // TODO: handle errors
        #![allow(unused_must_use)]

        // --- 2. Ensure the network to be removed exists.
        if !Self::if_subnet_exist(netuid) {
            return 0;
        }

        SubnetNames::<T>::remove(netuid);
        MaxWeightAge::<T>::remove(netuid);
        Name::<T>::clear_prefix(netuid, u32::MAX, None);
        Address::<T>::clear_prefix(netuid, u32::MAX, None);
        Metadata::<T>::clear_prefix(netuid, u32::MAX, None);
        Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        DelegationFee::<T>::clear_prefix(netuid, u32::MAX, None);

        // Remove consnesus vectors
        Weights::<T>::clear_prefix(netuid, u32::MAX, None);

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

        RegistrationBlock::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 2. Erase subnet parameters.
        Founder::<T>::remove(netuid);
        FounderShare::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        IncentiveRatio::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        MaxAllowedWeights::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        MinStake::<T>::remove(netuid);
        SelfVote::<T>::remove(netuid);
        SubnetEmission::<T>::remove(netuid);
        Tempo::<T>::remove(netuid);
        TrustRatio::<T>::remove(netuid);
        VoteModeSubnet::<T>::remove(netuid);

        // Adjust the total number of subnets. and remove the subnet from the list of subnets.
        N::<T>::remove(netuid);
        TotalSubnets::<T>::mutate(|val| *val -= 1);
        SubnetGaps::<T>::mutate(|subnets| subnets.insert(netuid));

        // --- 4. Emit the event.
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

        // --3. Ensure that the subnet is not in a `Vote` mode.
        // Update by founder can be executed only in `Authority` mode.
        ensure!(
            VoteModeSubnet::<T>::get(netuid) == VoteMode::Authority,
            Error::<T>::InvalidVoteMode
        );

        // -4. Apply the changeset.
        changeset.apply(netuid)?;

        // --- 5. Ok and done.
        Ok(())
    }

    // /// Empties out all:
    // /// emission, dividends, incentives, trust on the specific netuid.
    // fn deactivate_subnet(netuid: u16) {
    //     let module_count = N::<T>::get(netuid) as usize;
    //     let zeroed = vec![0; module_count];

    //     SubnetEmission::<T>::insert(netuid, 0);

    //     Active::<T>::insert(netuid, vec![true; module_count]);
    //     Consensus::<T>::insert(netuid, &zeroed);
    //     Dividends::<T>::insert(netuid, &zeroed);
    //     Emission::<T>::insert(netuid, vec![0; module_count]);
    //     Incentive::<T>::insert(netuid, &zeroed);
    //     PruningScores::<T>::insert(netuid, &zeroed);
    //     Rank::<T>::insert(netuid, &zeroed);
    //     Trust::<T>::insert(netuid, &zeroed);
    //     ValidatorPermits::<T>::insert(netuid, vec![false; module_count]);
    //     ValidatorTrust::<T>::insert(netuid, &zeroed);
    // }

    // ---------------------------------
    // Setters
    // ---------------------------------

    fn set_max_allowed_uids(netuid: u16, mut max_allowed_uids: u16) {
        let n: u16 = N::<T>::get(netuid);
        if max_allowed_uids < n {
            // limit it at 256 at a time
            let mut remainder_n: u16 = n - max_allowed_uids;
            let max_remainder = 256;
            if remainder_n > max_remainder {
                // remove the modules in small amounts, as this can be a heavy load on the chain
                remainder_n = max_remainder;
                max_allowed_uids = n - remainder_n;
            }
            // remove the modules by adding the to the deregister queue
            for i in 0..remainder_n {
                let next_uid: u16 = n - 1 - i;
                Self::remove_module(netuid, next_uid);
            }
        }

        MaxAllowedUids::<T>::insert(netuid, max_allowed_uids);
    }

    pub fn set_last_update_for_uid(netuid: u16, uid: u16, last_update: u64) {
        let mut updated_last_update_vec = LastUpdate::<T>::get(netuid);
        if (uid as usize) < updated_last_update_vec.len() {
            updated_last_update_vec[uid as usize] = last_update;
            LastUpdate::<T>::insert(netuid, updated_last_update_vec);
        }
    }

    // ---------------------------------
    // Getters
    // ---------------------------------

    // Gets the subnet with the lowest emisison value, uf there is one.
    // Used in the deregistraiton logic, lowest emission subnets get deregistered first.
    pub fn get_lowest_emission_netuid() -> Option<u16> {
        SubnetEmission::<T>::iter()
            .min_by_key(|(_, emission)| *emission)
            .map(|(netuid, _)| netuid)
    }

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

        let founder_emission_ratio: I64F64 =
            I64F64::from_num(founder_share.min(100)) / I64F64::from_num(100);
        let founder_emission =
            (founder_emission_ratio * I64F64::from_num(token_emission)).to_num::<u64>();
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
            total_stake_from += ownership;
        }

        // add the module itself, if it has stake of its own
        if total_stake_from == I64F64::from_num(0) {
            ownership_vector.push((module_key.clone(), I64F64::from_num(0)));
        } else {
            ownership_vector =
                ownership_vector.into_iter().map(|(k, v)| (k, v / total_stake_from)).collect();
        }

        ownership_vector
    }

    pub fn is_key_registered_on_any_network(key: &T::AccountId) -> bool {
        Self::netuids().iter().any(|&netuid| Uids::<T>::contains_key(netuid, key))
    }

    pub fn is_registered(network: Option<u16>, key: &T::AccountId) -> bool {
        match network {
            Some(netuid) => Uids::<T>::contains_key(netuid, key),
            None => {
                let mut is_registered = false;
                for netuid in N::<T>::iter_keys() {
                    if Uids::<T>::contains_key(netuid, key) {
                        is_registered = true;
                        break;
                    }
                }
                is_registered
            }
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
