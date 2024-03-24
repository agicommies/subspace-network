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

        log::debug!("block_step for block: {block_number:?}");

        for (netuid, tempo) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            let registration_this_interval = Self::get_registrations_this_interval(netuid);

            // adjust registrations parameters
            Self::adjust_registration(netuid, block_number, registration_this_interval);

            let new_queued_emission: u64 = Self::calculate_network_emission(netuid);
            PendingEmission::<T>::mutate(netuid, |queued| *queued += new_queued_emission);
            log::debug!("netuid_i: {netuid:?} queued_emission: +{new_queued_emission:?} ");
            if Self::blocks_until_next_epoch(netuid, tempo, block_number) > 0 {
                continue;
            }
            let emission_to_drain: u64 = PendingEmission::<T>::get(netuid);
            let subnet_stake: I64F64 = I64F64::from_num(Self::get_total_subnet_stake(netuid));
            let total_stake: I64F64 = I64F64::from_num(Self::total_stake());
            let threshold = SubnetStakeThreshold::<T>::get();
            if threshold <= (subnet_stake / total_stake) * 100 {
                if netuid == 0 {
                    Self::linear_epoch(netuid, emission_to_drain)
                } else {
                    Self::yuma_epoch(netuid, emission_to_drain)
                }
            }
            PendingEmission::<T>::insert(netuid, 0);
        }
    }

    pub fn yuma_epoch(netuid: u16, token_emission: u64) {
        // Get subnetwork size.
        let n: u16 = Self::get_subnet_n(netuid);
        log::trace!("n: {:?}", n);

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!("current_block: {:?}", current_block);

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff(netuid) as u64;
        log::trace!("activity_cutoff: {:?}", activity_cutoff);

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update(netuid);
        log::trace!("Last update: {:?}", &last_update);

        // Inactive mask.
        let inactive: Vec<bool> = last_update
            .iter()
            .map(|updated| *updated + activity_cutoff < current_block)
            .collect();
        log::trace!("Inactive: {:?}", inactive.clone());

        // Logical negation of inactive.
        let active: Vec<bool> = inactive.iter().map(|&b| !b).collect();

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration(netuid);
        log::trace!("Block at registration: {:?}", &block_at_registration);

        // ===========
        // == Stake ==
        // ===========

        let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
        for (uid_i, hotkey) in
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
        {
            hotkeys.push((uid_i, hotkey));
        }
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in hotkeys.iter() {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        // range: I32F32(0, 1)
        log::trace!("S: {:?}", &stake);

        // =======================
        // == Validator permits ==
        // =======================

        // Get current validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit(netuid);
        log::trace!("validator_permits: {:?}", validator_permits);

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators(netuid);
        log::trace!("max_allowed_validators: {:?}", max_allowed_validators);

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk(&stake, max_allowed_validators as usize);
        log::trace!("new_validator_permits: {:?}", new_validator_permits);

        // ==================
        // == Active Stake ==
        // ==================

        let mut active_stake: Vec<I32F32> = stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&validator_forbids, &mut active_stake);

        // Normalize active stake.
        inplace_normalize(&mut active_stake);
        log::trace!("S:\n{:?}\n", &active_stake);

        // =============
        // == Weights ==
        // =============

        // Access network weights row unnormalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse(netuid);
        // log::trace!( "W: {:?}", &weights );

        // Mask weights that are not from permitted validators.
        weights = mask_rows_sparse(&validator_forbids, &weights);
        // log::trace!( "W (permit): {:?}", &weights );

        // Remove self-weight by masking diagonal.
        weights = mask_diag_sparse(&weights);
        // log::trace!( "W (permit+diag): {:?}", &weights );

        // Remove weights referring to deregistered neurons.
        weights = vec_mask_sparse_matrix(
            &weights,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        // log::trace!( "W (permit+diag+outdate): {:?}", &weights );

        // Normalize remaining weights.
        inplace_row_normalize_sparse(&mut weights);
        // log::trace!( "W (mask+norm): {:?}", &weights );

        // ================================
        // == Consensus, Validator Trust ==
        // ================================

        // Compute preranks: r_j = SUM(i) w_ij * s_i
        let preranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        // log::trace!( "R (before): {:?}", &preranks );

        // Clip weights at majority consensus
        let kappa: I32F32 = Self::get_float_kappa(netuid); // consensus majority ratio, e.g. 51%.
        let consensus: Vec<I32F32> = weighted_median_col_sparse(&active_stake, &weights, n, kappa);
        log::trace!("C: {:?}", &consensus);

        weights = col_clip_sparse(&weights, &consensus);
        // log::trace!( "W: {:?}", &weights );

        let validator_trust: Vec<I32F32> = row_sum_sparse(&weights);
        log::trace!("Tv: {:?}", &validator_trust);

        // =============================
        // == Ranks, Trust, Incentive ==
        // =============================

        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        let mut ranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        // log::trace!( "R (after): {:?}", &ranks );

        // Compute server trust: ratio of rank after vs. rank before.
        let trust: Vec<I32F32> = vecdiv(&ranks, &preranks); // range: I32F32(0, 1)
        log::trace!("T: {:?}", &trust);

        inplace_normalize(&mut ranks); // range: I32F32(0, 1)
        let incentive: Vec<I32F32> = ranks.clone();
        log::trace!("I (=R): {:?}", &incentive);

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse(netuid);
        // log::trace!( "B: {:?}", &bonds );

        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix(
            &bonds,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        // log::trace!( "B (outdatedmask): {:?}", &bonds );

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse(&mut bonds, n);
        // log::trace!( "B (mask+norm): {:?}", &bonds );

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = row_hadamard_sparse(&weights, &active_stake); // ΔB = W◦S (outdated W masked)
                                                                                                     // log::trace!( "ΔB: {:?}", &bonds_delta );

        // Normalize bonds delta.
        inplace_col_normalize_sparse(&mut bonds_delta, n); // sum_i b_ij = 1
                                                           // log::trace!( "ΔB (norm): {:?}", &bonds_delta );

        // Compute bonds moving average.
        let bonds_moving_average: I64F64 =
            I64F64::from_num(Self::get_bonds_moving_average(netuid)) / I64F64::from_num(1_000_000);
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        let mut ema_bonds: Vec<Vec<(u16, I32F32)>> = mat_ema_sparse(&bonds_delta, &bonds, alpha);

        // Normalize EMA bonds.
        inplace_col_normalize_sparse(&mut ema_bonds, n); // sum_i b_ij = 1
                                                         // log::trace!( "emaB: {:?}", &ema_bonds );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends: Vec<I32F32> = matmul_transpose_sparse(&ema_bonds, &incentive);
        inplace_normalize(&mut dividends);
        log::trace!("D: {:?}", &dividends);

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> =
            incentive.iter().zip(dividends.clone()).map(|(ii, di)| ii + di).collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&active_stake) {
                // no active stake
                normalized_validator_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission = stake.clone();
            } else {
                normalized_validator_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission = active_stake.clone();
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        let server_emission: Vec<I96F32> = normalized_server_emission
            .iter()
            .map(|se: &I32F32| I96F32::from_num(*se) * float_rao_emission)
            .collect();
        let server_emission: Vec<u64> =
            server_emission.iter().map(|e: &I96F32| e.to_num::<u64>()).collect();

        let validator_emission: Vec<I96F32> = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| I96F32::from_num(*ve) * float_rao_emission)
            .collect();
        let validator_emission: Vec<u64> =
            validator_emission.iter().map(|e: &I96F32| e.to_num::<u64>()).collect();

        // Only used to track emission in storage.
        let combined_emission: Vec<I96F32> = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| I96F32::from_num(*ce) * float_rao_emission)
            .collect();
        let combined_emission: Vec<u64> =
            combined_emission.iter().map(|e: &I96F32| e.to_num::<u64>()).collect();

        log::trace!("nSE: {:?}", &normalized_server_emission);
        log::trace!("SE: {:?}", &server_emission);
        log::trace!("nVE: {:?}", &normalized_validator_emission);
        log::trace!("VE: {:?}", &validator_emission);
        log::trace!("nCE: {:?}", &normalized_combined_emission);
        log::trace!("CE: {:?}", &combined_emission);

        // Set pruning scores using combined emission scores.
        let pruning_scores: Vec<I32F32> = normalized_combined_emission.clone();
        log::trace!("P: {:?}", &pruning_scores);

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = combined_emission.clone();
        let cloned_ranks: Vec<u16> =
            ranks.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> =
            trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> =
            consensus.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> =
            incentive.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> =
            dividends.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_pruning_scores: Vec<u16> = vec_max_upscale_to_u16(&pruning_scores);
        let cloned_validator_trust: Vec<u16> = validator_trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        Active::<T>::insert(netuid, active.clone());
        Emission::<T>::insert(netuid, cloned_emission);
        Rank::<T>::insert(netuid, cloned_ranks);
        Trust::<T>::insert(netuid, cloned_trust);
        Consensus::<T>::insert(netuid, cloned_consensus);
        Incentive::<T>::insert(netuid, cloned_incentive);
        Dividends::<T>::insert(netuid, cloned_dividends);
        PruningScores::<T>::insert(netuid, cloned_pruning_scores);
        ValidatorTrust::<T>::insert(netuid, cloned_validator_trust);
        ValidatorPermit::<T>::insert(netuid, new_validator_permits.clone());

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale_sparse(&mut ema_bonds, n);
        for i in 0..n {
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                let new_bonds_row: Vec<(u16, u16)> = ema_bonds[i as usize]
                    .iter()
                    .map(|(j, value)| (*j, fixed_proportion_to_u16(*value)))
                    .collect();
                Bonds::<T>::insert(netuid, i, new_bonds_row);
            } else if validator_permits[i as usize] {
                // Only overwrite the intersection.
                let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                Bonds::<T>::insert(netuid, i, new_empty_bonds_row);
            }
        }

        // Emission tuples ( hotkeys, server_emission, validator_emission )
        let mut result: Vec<(T::AccountId, u64, u64)> = vec![];
        for (uid_i, hotkey) in hotkeys.iter() {
            result.push((
                hotkey.clone(),
                server_emission[*uid_i as usize],
                validator_emission[*uid_i as usize],
            ));
        }
        result
    }

    /// This function acts as the main function of the entire blockchain reward distribution.
    /// It calculates the dividends, the incentive, the weights, the bonds,
    /// the trust and the emission for the epoch.
    pub fn linear_epoch(netuid: u16, token_emission: u64) {
        // get the network parameters
        let global_params = Self::global_params();
        let subnet_params = Self::subnet_params(netuid);

        // get the amount of modules
        let n: u16 = Self::get_subnet_n(netuid);
        let current_block: u64 = Self::get_current_block_number();

        // if there are no modules, then return
        if n == 0 {
            return;
        }

        // FOUNDER DIVIDENDS
        let founder_key = Self::get_founder(netuid);
        let (token_emission, founder_emission) =
            Self::calculate_founder_emission(netuid, token_emission, &founder_key);

        // STAKE
        let uid_key_tuples: Vec<(u16, T::AccountId)> = Self::get_uid_key_tuples(netuid);
        let total_stake_u64: u64 = Self::get_total_subnet_stake(netuid).max(1);

        let max_stake = subnet_params.max_stake;

        let stake_u64: Vec<u64> = uid_key_tuples
            .iter()
            .map(|(_, key)| Self::get_stake_for_key(netuid, key).min(max_stake))
            .collect();

        // Clip it to the max stake
        let stake_f64: Vec<I64F64> = stake_u64
            .iter()
            .map(|x| I64F64::from_num(*x) / I64F64::from_num(total_stake_u64))
            .collect();

        let mut stake: Vec<I32F32> = stake_f64.iter().map(|x| I32F32::from_num(*x)).collect();

        // Normalize stake.
        inplace_normalize(&mut stake);

        // WEIGHTS
        let weights: Vec<Vec<(u16, I32F32)>> = Self::process_weights(
            netuid,
            n,
            &global_params,
            &subnet_params,
            current_block,
            &stake_f64,
            total_stake_u64,
        );

        // INCENTIVE
        // see if this shit needs to be mut
        let mut incentive: Vec<I32F32> =
            Self::compute_incentive(&weights, &stake, &uid_key_tuples, n);

        // TRUST
        // trust that acts as a multiplier for the incentive
        let trust_ratio: u16 = Self::get_trust_ratio(netuid);
        if trust_ratio > 0 {
            let trust_share: I32F32 = I32F32::from_num(trust_ratio) / I32F32::from_num(100);
            let incentive_share: I32F32 = I32F32::from_num(1.0).saturating_sub(trust_share);
            let trust = Self::compute_trust(&weights, &stake, &subnet_params, n);

            incentive = incentive
                .iter()
                .zip(trust.iter())
                .map(|(inc, tru)| (inc * incentive_share) + (tru * trust_share))
                .collect();

            // save the trust into the trust vector
            Trust::<T>::insert(
                netuid,
                trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>(),
            );
        }

        // store the incentive
        let cloned_incentive: Vec<u16> =
            incentive.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        Incentive::<T>::insert(netuid, cloned_incentive);

        //  BONDS
        let bonds: Vec<Vec<(u16, I32F32)>> = Self::compute_bonds_delta(&weights, &stake);

        // DIVIDENDS
        let (fixed_dividends, dividends) =
            Self::compute_dividends(&bonds, &incentive, &uid_key_tuples);
        Dividends::<T>::insert(netuid, fixed_dividends);

        // EMISSION
        Self::process_emission(
            &incentive,
            &dividends,
            token_emission,
            netuid,
            founder_emission,
            &founder_key,
            &uid_key_tuples,
        );
    }

    fn calculate_emission_ratios(
        incentive: &[I32F32],
        dividends: &[I32F32],
        token_emission: u64,
        netuid: u16,
    ) -> (Vec<I64F64>, Vec<I64F64>) {
        let incentive_ratio: I64F64 =
            I64F64::from_num(Self::get_incentive_ratio(netuid) as u64) / I64F64::from_num(100);
        let dividend_ratio: I64F64 = I64F64::from_num(1.0) - incentive_ratio;

        let incentive_emission_float: Vec<I64F64> = incentive
            .iter()
            .map(|&x| I64F64::from_num(x) * I64F64::from_num(token_emission) * incentive_ratio)
            .collect();
        let dividends_emission_float: Vec<I64F64> = dividends
            .iter()
            .map(|&x| I64F64::from_num(x) * I64F64::from_num(token_emission) * dividend_ratio)
            .collect();

        (incentive_emission_float, dividends_emission_float)
    }

    fn calculate_emissions(
        incentive_emission_float: &[I64F64],
        dividends_emission_float: &[I64F64],
        founder_emission: u64,
        netuid: u16,
        founder_key: &T::AccountId,
        uid_key_tuples: &[(u16, T::AccountId)],
    ) -> Vec<u64> {
        let n = incentive_emission_float.len();
        let mut incentive_emission: Vec<u64> =
            incentive_emission_float.iter().map(|e| e.to_num::<u64>()).collect();
        let dividends_emission: Vec<u64> =
            dividends_emission_float.iter().map(|e| e.to_num::<u64>()).collect();

        let burn_amount_per_epoch: u64 = Self::get_burn_per_epoch(netuid);

        let founder_uid = Self::get_uid_for_key(netuid, founder_key);
        incentive_emission[founder_uid as usize] =
            incentive_emission[founder_uid as usize].saturating_add(founder_emission);

        let mut emission: Vec<u64> = vec![0; n];

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
                        (I64F64::from_num(total_owner_dividends_emission) * *delegate_ratio)
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

    fn process_emission(
        incentive: &[I32F32],
        dividends: &[I32F32],
        token_emission: u64,
        netuid: u16,
        founder_emission: u64,
        founder_key: &T::AccountId,
        uid_key_tuples: &[(u16, T::AccountId)],
    ) {
        let (incentive_emission_float, dividends_emission_float) =
            Self::calculate_emission_ratios(incentive, dividends, token_emission, netuid);

        let emission = Self::calculate_emissions(
            &incentive_emission_float,
            &dividends_emission_float,
            founder_emission,
            netuid,
            founder_key,
            uid_key_tuples,
        );

        Emission::<T>::insert(netuid, emission);
    }

    fn compute_dividends(
        bonds: &[Vec<(u16, I32F32)>],
        incentive: &[I32F32],
        uid_key_tuples: &[(u16, T::AccountId)],
    ) -> (Vec<u16>, Vec<I32F32>) {
        let n = incentive.len();
        let mut dividends: Vec<I32F32> = vec![I32F32::from_num(0.0); n];

        for (i, sparse_row) in bonds.iter().enumerate() {
            for (j, value) in sparse_row.iter() {
                dividends[i] += incentive[*j as usize] * *value;
            }
        }

        if dividends.iter().all(|&x| x == I32F32::from_num(0.0)) {
            for (uid_i, _) in uid_key_tuples.iter() {
                dividends[*uid_i as usize] = I32F32::from_num(1.0);
            }
        }

        inplace_normalize(&mut dividends);

        let fixed_dividends: Vec<u16> =
            dividends.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect();

        (fixed_dividends, dividends)
    }

    fn compute_bonds_delta(
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
    ) -> Vec<Vec<(u16, I32F32)>> {
        let n = weights.len();
        let mut bonds: Vec<Vec<(u16, I32F32)>> = weights.to_vec();
        let mut col_sum: Vec<I32F32> = vec![I32F32::from_num(0.0); n];

        for (i, sparse_row) in bonds.iter_mut().enumerate() {
            for (j, value) in sparse_row.iter_mut() {
                *value *= stake[i];
                col_sum[*j as usize] += *value;
            }
        }

        for sparse_row in bonds.iter_mut() {
            for (j, value) in sparse_row.iter_mut() {
                if col_sum[*j as usize] > I32F32::from_num(0.0) {
                    *value /= col_sum[*j as usize];
                }
            }
        }

        bonds
    }

    fn compute_trust(
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
        subnet_params: &SubnetParams<T>,
        n: u16,
    ) -> Vec<I32F32> {
        let mut trust = vec![I32F32::from_num(0.0); n as usize];
        for (i, weights_i) in weights.iter().enumerate() {
            for (j, weight_ij) in weights_i.iter() {
                if *weight_ij > 0 && stake[i] > I32F32::from_num(subnet_params.min_stake) {
                    trust[*j as usize] += I32F32::from_num(1.0);
                }
            }
        }
        inplace_normalize(&mut trust);
        trust
    }

    fn compute_incentive(
        weights: &[Vec<(u16, I32F32)>],
        stake: &[I32F32],
        uid_key_tuples: &[(u16, T::AccountId)],
        n: u16,
    ) -> Vec<I32F32> {
        let mut incentive: Vec<I32F32> = vec![I32F32::from_num(0.0); n as usize];

        for (i, sparse_row) in weights.iter().enumerate() {
            for (j, value) in sparse_row.iter() {
                incentive[*j as usize] += stake[i] * value;
            }
        }

        if is_zero(&incentive) {
            for (uid_i, _key) in uid_key_tuples.iter() {
                incentive[*uid_i as usize] = I32F32::from_num(1.0);
            }
        }

        inplace_normalize(&mut incentive);
        incentive
    }

    fn get_current_weight_age(last_update_vector: &[u64], current_block: u64, uid_i: u16) -> u64 {
        current_block.saturating_sub(last_update_vector[uid_i as usize])
    }

    #[allow(clippy::too_many_arguments)]
    fn check_weight_validity(
        weight_age: u64,
        subnet_params: &SubnetParams<T>,
        weights_i: &[(u16, u16)],
        stake_f64: &[I64F64],
        total_stake_u64: u64,
        min_weight_stake_f64: I64F64,
        n: u16,
        uid_i: u16,
    ) -> (bool, Vec<(u16, u16)>) {
        let mut valid_weights = Vec::new();

        if weight_age > subnet_params.max_weight_age
            || weights_i.len() < subnet_params.min_allowed_weights as usize
        {
            return (true, valid_weights);
        }

        for (pos, (uid_j, weight_ij)) in weights_i.iter().enumerate() {
            if (pos as u16) > subnet_params.max_allowed_weights || *uid_j >= n {
                return (true, valid_weights);
            }

            let weight_f64 = I64F64::from_num(*weight_ij) / I64F64::from_num(u16::MAX);
            let weight_stake =
                (stake_f64[uid_i as usize] * weight_f64) * I64F64::from_num(total_stake_u64);

            if weight_stake > min_weight_stake_f64 {
                valid_weights.push((*uid_j, *weight_ij));
            } else {
                return (true, valid_weights);
            }
        }

        (false, valid_weights)
    }

    fn process_weights(
        netuid: u16,
        n: u16,
        global_params: &GlobalParams<T>,
        subnet_params: &SubnetParams<T>,
        current_block: u64,
        stake_f64: &[I64F64],
        total_stake_u64: u64,
    ) -> Vec<Vec<(u16, I32F32)>> {
        let last_update_vector = Self::get_last_update(netuid);
        let min_weight_stake_f64 = I64F64::from_num(global_params.min_weight_stake);
        let mut weights: Vec<Vec<(u16, u16)>> = vec![vec![]; n as usize];

        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            let weight_age =
                Self::get_current_weight_age(&last_update_vector, current_block, uid_i);
            let (weight_changed, valid_weights) = Self::check_weight_validity(
                weight_age,
                subnet_params,
                &weights_i,
                stake_f64,
                total_stake_u64,
                min_weight_stake_f64,
                n,
                uid_i,
            );

            weights[uid_i as usize] = valid_weights;
            if weight_changed {
                <Weights<T>>::insert(netuid, uid_i, weights[uid_i as usize].clone());
            }
        }

        let mut weights: Vec<Vec<(u16, I32F32)>> = weights
            .iter()
            .map(|x| {
                x.iter().map(|(uid, weight)| (*uid, u16_proportion_to_fixed(*weight))).collect()
            })
            .collect();

        weights = mask_diag_sparse(&weights);
        inplace_row_normalize_sparse(&mut weights);

        weights
    }

    fn calculate_founder_emission(
        netuid: u16,
        mut token_emission: u64,
        founder_key: &T::AccountId,
    ) -> (u64, u64) {
        let is_founder_registered = Self::key_registered(netuid, founder_key);
        if !is_founder_registered {
            return (token_emission, 0);
        }

        let founder_share: u16 = Self::get_founder_share(netuid);
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

    pub fn adjust_registration(netuid: u16, block_number: u64, registrations_this_interval: u16) {
        let target_registrations_interval = Self::get_target_registrations_interval();
        if block_number % u64::from(target_registrations_interval) == 0 {
            let current_burn = Self::get_burn(netuid);
            let target_registrations_per_interval = Self::get_target_registrations_per_interval();

            let adjusted_burn = Self::adjust_burn(
                current_burn,
                registrations_this_interval,
                target_registrations_per_interval,
            );

            Self::set_burn(netuid, adjusted_burn);

            // reset the registrations
            Self::set_registrations_this_interval(netuid, 0);
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

    pub fn get_float_kappa(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_kappa(netuid)) / I32F32::from_num(u16::MAX)
    }
}
