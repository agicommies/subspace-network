use super::*;
use crate::math::*;
use frame_support::storage::{IterableStorageDoubleMap, IterableStorageMap};
use sp_std::vec;
use substrate_fixed::types::{I110F18, I32F32, I64F64};

impl<T: Config> Pallet<T> {
    pub fn block_step() {
        let block_number: u64 = Self::get_current_block_number();
        RegistrationsPerBlock::<T>::mutate(|val| *val = 0);

        let registration_this_interval = Self::get_registrations_this_interval();

        // adjust registrations parameters
        Self::adjust_registration(block_number, registration_this_interval);

        log::debug!("block_step for block: {:?} ", block_number);
        for (netuid, tempo) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            let new_queued_emission: u64 = Self::calculate_network_emission(netuid);
            PendingEmission::<T>::mutate(netuid, |queued| *queued += new_queued_emission);
            log::debug!(
                "netuid_i: {:?} queued_emission: +{:?} ",
                netuid,
                new_queued_emission
            );
            if Self::blocks_until_next_epoch(netuid, tempo, block_number) > 0 {
                continue;
            }
            let emission_to_drain: u64 = PendingEmission::<T>::get(netuid);
            Self::epoch(netuid, emission_to_drain);
            PendingEmission::<T>::insert(netuid, 0);
        }
    }

    pub fn epoch(netuid: u16, token_emission: u64) {
        let global_params = Self::global_params();
        let subnet_params = Self::subnet_params(netuid);

        let n: u16 = Self::get_subnet_n(netuid);
        let current_block: u64 = Self::get_current_block_number();

        if n == 0 {
            return;
        }

        // =======================
        // == Founder Dividends ==
        // =======================
        let (token_emission, founder_emission) =
            Self::calculate_founder_emission(netuid, token_emission);

        // ===========
        // == Stake ==
        // ===========

        let uid_key_tuples: Vec<(u16, T::AccountId)> = Self::get_uid_key_tuples(netuid);
        let total_stake_u64: u64 = Self::get_total_subnet_stake(netuid).max(1);

        let stake_u64: Vec<u64> =
            Self::get_stake_vector(netuid, &uid_key_tuples, subnet_params.max_stake);
        let stake_f64: Vec<I64F64> = Self::normalize_stake_vector(&stake_u64, total_stake_u64);
        let mut stake: Vec<I32F32> = stake_f64.iter().map(|x| I32F32::from_num(*x)).collect();

        // normalize
        inplace_normalize(&mut stake);

        // =============
        // == Weights (N x N) Sparsified ==
        // =============

        let last_update_vector: Vec<u64> = Self::get_last_update(netuid);
        let mut weights: Vec<Vec<(u16, u16)>> = Self::get_weights(netuid, n);
        Self::update_weights(
            global_params,
            n,
            netuid,
            &mut weights,
            &last_update_vector,
            &stake_f64,
            current_block,
            &subnet_params,
        );

        let weights: Vec<Vec<(u16, I32F32)>> = Self::convert_weights_to_fixed(&weights);
        let weights = Self::weight_normalization(weights);

        // =============================
        // ==  Incentive ==
        // =============================

        let mut incentive: Vec<I32F32> = Self::calculate_incentive(&weights, &stake, n);

        if is_zero(&incentive) {
            incentive = Self::even_split_incentive(&uid_key_tuples, n);
        }

        // =================================
        // == TRUST ==
        // =================================

        let incentive = Self::calculate_trust_and_update_incentive(
            netuid,
            &weights,
            &stake,
            &incentive,
            I32F32::from_num::<u64>(subnet_params.min_stake.into()),
        );
        // store the incentive
        let cloned_incentive: Vec<u16> = incentive
            .0
            .iter()
            .chain(incentive.1.iter())
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect();
        Incentive::<T>::insert(netuid, cloned_incentive);

        // =================================
        // == Calculate Bonds ==
        // =================================

        let bonds: Vec<Vec<(u16, I32F32)>> = Self::calculate_bonds(&weights, &stake, n as usize);

        // =================================
        // == Dividends ==
        // =================================

        let dividends: Vec<I32F32> =
            Self::calculate_dividends(&bonds, &incentive.0, &uid_key_tuples);

        // =================================
        // == Emission ==
        // =================================

        let emission = Self::calculate_emission(
            netuid,
            &incentive.0,
            &dividends,
            token_emission,
            founder_emission,
            Self::get_burn_per_epoch(netuid),
            &uid_key_tuples,
            Self::key_registered(netuid, &Self::get_founder(netuid)),
            &Self::get_founder(netuid),
        );

        Emission::<T>::insert(netuid, emission);
    }

    fn calculate_emission(
        netuid: u16,
        incentive: &[I32F32],
        dividends: &[I32F32],
        token_emission: u64,
        founder_emission: u64,
        burn_amount_per_epoch: u64,
        uid_key_tuples: &Vec<(u16, T::AccountId)>,
        is_founder_registered: bool,
        founder_key: &T::AccountId,
    ) -> Vec<u64> {
        let incentive_ratio: I64F64 =
            I64F64::from_num(Self::get_incentive_ratio(netuid) as u64) / I64F64::from_num(100);
        let dividend_ratio: I64F64 = I64F64::from_num(1.0) - incentive_ratio;

        let incentive_emission_float: Vec<I64F64> = incentive
            .iter()
            .map(|x| I64F64::from_num(*x) * I64F64::from_num(token_emission) * incentive_ratio)
            .collect();
        let dividends_emission_float: Vec<I64F64> = dividends
            .iter()
            .map(|x| I64F64::from_num(*x) * I64F64::from_num(token_emission) * dividend_ratio)
            .collect();

        let mut incentive_emission: Vec<u64> =
            incentive_emission_float.iter().map(|e: &I64F64| e.to_num::<u64>()).collect();
        let dividends_emission: Vec<u64> =
            dividends_emission_float.iter().map(|e: &I64F64| e.to_num::<u64>()).collect();

        if is_founder_registered {
            let founder_uid = Self::get_uid_for_key(netuid, founder_key);
            incentive_emission[founder_uid as usize] =
                incentive_emission[founder_uid as usize].saturating_add(founder_emission);
        }

        let mut emission: Vec<u64> = vec![0; incentive.len()];

        for (module_uid, module_key) in uid_key_tuples.iter() {
            let mut owner_emission_incentive: u64 = incentive_emission[*module_uid as usize];
            let mut owner_dividends_emission: u64 = dividends_emission[*module_uid as usize];
            let owner_emission: u64 = owner_emission_incentive + owner_dividends_emission;

            if burn_amount_per_epoch > owner_emission {
                let burn_into_stake: u64 = burn_amount_per_epoch.saturating_sub(owner_emission);

                if burn_into_stake > 0 {
                    Self::decrease_stake(netuid, module_key, module_key, burn_into_stake);
                }

                continue;
            }

            if burn_amount_per_epoch > owner_emission_incentive {
                owner_emission_incentive = 0;
                let left_burn_amount_per_epoch =
                    burn_amount_per_epoch.saturating_sub(owner_emission_incentive);
                owner_dividends_emission =
                    owner_dividends_emission.saturating_sub(left_burn_amount_per_epoch);
            } else {
                owner_emission_incentive =
                    owner_emission_incentive.saturating_sub(burn_amount_per_epoch);
            }

            emission[*module_uid as usize] = owner_emission_incentive + owner_dividends_emission;

            if owner_dividends_emission > 0 {
                let ownership_vector: Vec<(T::AccountId, I64F64)> =
                    Self::get_ownership_ratios(netuid, module_key);
                let delegation_fee = Self::get_delegation_fee(netuid, module_key);
                let total_owner_dividends_emission: u64 = owner_dividends_emission;

                for (delegate_key, delegate_ratio) in ownership_vector.iter() {
                    if delegate_key == module_key {
                        continue;
                    }

                    let dividends_from_delegate: u64 =
                        (I64F64::from_num(total_owner_dividends_emission) * delegate_ratio)
                            .to_num::<u64>();
                    let to_module: u64 = delegation_fee.mul_floor(dividends_from_delegate);
                    let to_delegate: u64 = dividends_from_delegate.saturating_sub(to_module);
                    Self::increase_stake(netuid, delegate_key, module_key, to_delegate);
                    owner_dividends_emission = owner_dividends_emission.saturating_sub(to_delegate);
                }
            }

            let owner_emission: u64 = owner_emission_incentive + owner_dividends_emission;

            if owner_emission > 0 {
                let profit_share_emissions: Vec<(T::AccountId, u64)> =
                    Self::get_profit_share_emissions(module_key.clone(), owner_emission);

                if !profit_share_emissions.is_empty() {
                    for (profit_share_key, profit_share_emission) in profit_share_emissions.iter() {
                        Self::increase_stake(
                            netuid,
                            profit_share_key,
                            module_key,
                            *profit_share_emission,
                        );
                    }
                } else {
                    Self::increase_stake(netuid, module_key, module_key, owner_emission);
                }
            }
        }

        emission
    }

    fn calculate_bonds(
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
        n: usize,
    ) -> Vec<Vec<(u16, I32F32)>> {
        let mut bonds: Vec<Vec<(u16, I32F32)>> = weights.to_vec();
        let mut col_sum: Vec<I32F32> = vec![I32F32::from_num(0.0); n];

        for (i, sparse_row) in bonds.iter_mut().enumerate() {
            for (j, value) in sparse_row.iter_mut() {
                *value *= stake[i]; // scale by stake
                col_sum[*j as usize] += *value; // sum the column
            }
        }

        // sum the votes per module
        for sparse_row in bonds.iter_mut() {
            for (j, value) in sparse_row.iter_mut() {
                if col_sum[*j as usize] > I32F32::from_num(0.0) {
                    *value /= col_sum[*j as usize];
                }
            }
        }

        bonds
    }

    fn calculate_dividends(
        bonds: &[Vec<(u16, I32F32)>],
        incentive: &[I32F32],
        uid_key_tuples: &Vec<(u16, T::AccountId)>,
    ) -> Vec<I32F32> {
        let mut dividends: Vec<I32F32> = vec![I32F32::from_num(0.0); incentive.len()];

        for (i, sparse_row) in bonds.iter().enumerate() {
            for (j, value) in sparse_row.iter() {
                dividends[i] += incentive[*j as usize] * value;
            }
        }

        // If emission is zero, do an even split.
          if is_zero(&dividends) {
        for (uid_i, _key) in uid_key_tuples.iter() {
            dividends[*uid_i as usize] = I32F32::from_num(1.0);
        }
    }

        inplace_normalize(&mut dividends);
        dividends
    }

    fn calculate_founder_emission(netuid: u16, mut token_emission: u64) -> (u64, u64) {
        let founder_key = Self::get_founder(netuid);
        let is_founder_registered = Self::key_registered(netuid, &founder_key);
        let mut founder_emission: u64 = 0;

        if is_founder_registered {
            let founder_share: u16 = Self::get_founder_share(netuid);
            if founder_share > 0 {
                let founder_emission_ratio: I64F64 =
                    I64F64::from_num(founder_share.min(100)) / I64F64::from_num(100);
                founder_emission =
                    (founder_emission_ratio * I64F64::from_num(token_emission)).to_num::<u64>();
                token_emission = token_emission.saturating_sub(founder_emission);
            }
        }
        (token_emission, founder_emission)
    }

    fn get_stake_vector(
        netuid: u16,
        uid_key_tuples: &Vec<(u16, T::AccountId)>,
        max_stake: u64,
    ) -> Vec<u64> {
        uid_key_tuples
            .iter()
            .map(|(_, key)| Self::get_stake_for_key(netuid, key).min(max_stake))
            .collect()
    }

    fn normalize_stake_vector(stake_u64: &[u64], total_stake_u64: u64) -> Vec<I64F64> {
        stake_u64
            .iter()
            .map(|x| I64F64::from_num(*x) / I64F64::from_num(total_stake_u64))
            .collect()
    }

    fn get_weights(netuid: u16, n: u16) -> Vec<Vec<(u16, u16)>> {
        let mut weights: Vec<Vec<(u16, u16)>> = vec![vec![]; n as usize];

        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            weights[uid_i as usize] = weights_i;
        }

        weights
    }

    fn update_weights(
        global_params: GlobalParams,
        n: u16,
        netuid: u16,
        weights: &mut Vec<Vec<(u16, u16)>>,
        last_update_vector: &[u64],
        stake_f64: &[I64F64],
        current_block: u64,
        subnet_params: &SubnetParams<T>,
    ) {
        let min_weight_stake_f64: I64F64 = I64F64::from_num(global_params.min_weight_stake);
        let total_stake_u64: u64 = Self::get_total_subnet_stake(netuid);

        for (uid_i, weights_i) in weights.iter_mut().enumerate() {
            let weight_age: u64 = current_block.saturating_sub(last_update_vector[uid_i]);

            if weight_age > subnet_params.max_weight_age
                || weights_i.len() < subnet_params.min_allowed_weights as usize
            {
                weights_i.clear();
            } else {
                weights_i.retain(|(uid_j, weight_ij)| {
                    if *uid_j < n && *weight_ij <= subnet_params.max_allowed_weights {
                        let weight_f64 = I64F64::from_num(*weight_ij) / I64F64::from_num(u16::MAX);
                        let weight_stake =
                            (stake_f64[uid_i] * weight_f64) * I64F64::from_num(total_stake_u64);
                        weight_stake > min_weight_stake_f64
                    } else {
                        false
                    }
                });
            }

            <Weights<T>>::insert(netuid, uid_i as u16, weights_i.clone());
        }
    }

    fn convert_weights_to_fixed(weights: &[Vec<(u16, u16)>]) -> Vec<Vec<(u16, I32F32)>> {
        weights
            .iter()
            .map(|x| {
                x.iter().map(|(uid, weight)| (*uid, u16_proportion_to_fixed(*weight))).collect()
            })
            .collect()
    }

    fn weight_normalization(weights: Vec<Vec<(u16, I32F32)>>) -> Vec<Vec<(u16, I32F32)>> {
        let weights = mask_diag_sparse(&weights);
        inplace_row_normalize_sparse(&mut weights.clone());
        weights
    }

    fn calculate_incentive(
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
        n: u16,
    ) -> Vec<I32F32> {
        let mut incentive: Vec<I32F32> = vec![I32F32::from_num(0.0); n as usize];

        for (i, sparse_row) in weights.iter().enumerate() {
            for (j, value) in sparse_row.iter() {
                incentive[*j as usize] += stake[i] * value;
            }
        }

        inplace_normalize(&mut incentive);
        incentive
    }

    fn even_split_incentive(uid_key_tuples: &[(u16, T::AccountId)], n: u16) -> Vec<I32F32> {
        let mut incentive: Vec<I32F32> = vec![I32F32::from_num(0.0); n as usize];
        for (uid_i, _key) in uid_key_tuples.iter() {
            incentive[*uid_i as usize] = I32F32::from_num(1.0);
        }
        incentive
    }

    fn calculate_trust_and_update_incentive(
        netuid: u16,
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
        incentive: &[I32F32],
        min_stake: I32F32,
    ) -> (Vec<I32F32>, Vec<I32F32>) {
        let trust_ratio: u16 = Self::get_trust_ratio(netuid);
        let n = incentive.len();

        if trust_ratio > 0 {
            let trust_share: I32F32 = I32F32::from_num(trust_ratio) / I32F32::from_num(100);
            let incentive_share: I32F32 = I32F32::from_num(1.0).saturating_sub(trust_share);
            let mut trust: Vec<I32F32> = vec![I32F32::from_num(0.0); n];

            for (i, weights_i) in weights.iter().enumerate() {
                for (j, weight_ij) in weights_i.iter() {
                    if *weight_ij > 0 && stake[i] > min_stake {
                        trust[*j as usize] += I32F32::from_num(1.0);
                    }
                }
            }

            inplace_normalize(&mut trust);
            let incentive: Vec<I32F32> = incentive
                .iter()
                .zip(trust.iter())
                .map(|(inc, tru)| (inc * incentive_share) + (tru * trust_share))
                .collect();

            Trust::<T>::insert(
                netuid,
                trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>(),
            );

            (incentive, trust)
        } else {
            (incentive.to_vec(), vec![I32F32::from_num(0.0); n])
        }
    }

    pub fn get_block_at_registration(netuid: u16) -> Vec<u64> {
        let n = Self::get_subnet_n(netuid) as usize;
        let mut block_at_registration: Vec<u64> = vec![0; n];

        for (module_uid, block) in block_at_registration.iter_mut().enumerate() {
            let module_uid = module_uid as u16;

            if Keys::<T>::contains_key(netuid, module_uid) {
                *block = Self::get_module_registration_block(netuid, module_uid);
            }
        }

        block_at_registration
    }

    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        if tempo == 0 {
            return 0;
        }
        (block_number + netuid as u64) % (tempo as u64)
    }

    pub fn get_ownership_ratios(
        netuid: u16,
        module_key: &T::AccountId,
    ) -> Vec<(T::AccountId, I64F64)> {
        let stake_from_vector: Vec<(T::AccountId, u64)> =
            Self::get_stake_from_vector(netuid, module_key);
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

    #[cfg(debug_assertions)]
    pub fn get_ownership_ratios_emission(
        netuid: u16,
        module_key: &T::AccountId,
        emission: u64,
    ) -> Vec<(T::AccountId, u64)> {
        let ownership_vector: Vec<(T::AccountId, I64F64)> =
            Self::get_ownership_ratios(netuid, module_key);
        let mut emission_vector: Vec<(T::AccountId, u64)> = Vec::new();

        for (k, v) in ownership_vector {
            let emission_for_delegate = (v * I64F64::from_num(emission)).to_num::<u64>();
            emission_vector.push((k, emission_for_delegate));
        }

        emission_vector
    }

    pub fn get_burn_per_epoch(netuid: u16) -> u64 {
        let n = Self::get_subnet_n(netuid);
        let token_emission: u64 = PendingEmission::<T>::get(netuid);
        let burn_rate: u16 = Self::get_burn_rate().min(100);
        let mut burn_amount_per_epoch: u64 = 0;
        // get the float and convert to u64token_emission
        if burn_rate > 0 {
            let burn_rate_float: I64F64 = (I64F64::from_num(burn_rate) / I64F64::from_num(100))
                * (I64F64::from_num(token_emission) / I64F64::from_num(n));
            burn_amount_per_epoch = burn_rate_float.to_num::<u64>();
        }
        burn_amount_per_epoch
    }

    pub fn adjust_registration(block_number: u64, registrations_this_interval: u16) {
        let target_registrations_interval = Self::get_target_registrations_interval();
        if block_number % u64::from(target_registrations_interval) == 0 {
            let current_burn = Self::get_burn();
            let target_registrations_per_interval = Self::get_target_registrations_per_interval();

            Self::set_burn(Self::adjust_burn(
                current_burn,
                registrations_this_interval,
                target_registrations_per_interval,
            ));

            RegistrationsThisInterval::<T>::mutate(|val| *val = 0);
        }
    }

    pub fn adjust_burn(
        current_burn: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_burn: I110F18 = I110F18::from_num(current_burn)
            * I110F18::from_num(registrations_this_interval + target_registrations_per_interval)
            / I110F18::from_num(
                target_registrations_per_interval + target_registrations_per_interval,
            );
        let alpha: I110F18 =
            I110F18::from_num(Self::get_adjustment_alpha()) / I110F18::from_num(u64::MAX);
        let next_value: I110F18 = alpha * I110F18::from_num(current_burn)
            + (I110F18::from_num(1.0) - alpha) * updated_burn;
        if next_value >= I110F18::from_num(Self::get_max_burn()) {
            Self::get_max_burn()
        } else if next_value <= I110F18::from_num(Self::get_min_burn()) {
            Self::get_min_burn()
        } else {
            next_value.to_num::<u64>()
        }
    }
}
