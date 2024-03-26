use core::marker::PhantomData;

use substrate_fixed::types::{I32F32, I64F64};

use crate::{math::*, Config, Keys, Pallet, Weights};

struct YumaCalc<T: Config> {
    n: u16,
    netuid: u16,
    /// Consensus majority ratio, e.g. 51%.
    kappa: I32F32,

    last_update: Vec<u64>,
    block_at_registration: Vec<u64>,

    weights: Vec<Vec<(u16, I32F32)>>,
    stake: Vec<I32F32>,
    active_stake: Vec<I32F32>,
    preranks: Vec<I32F32>,
    incentive: Vec<I32F32>,
    trust: Vec<I32F32>,

    validator_permits: Vec<bool>,
    validator_forbids: Vec<bool>,
    max_allowed_validators: usize,

    _pd: PhantomData<T>,
}

impl<T: Config> YumaCalc<T> {
    fn new(netuid: u16) -> Self {
        let validator_permits = Pallet::<T>::get_validator_permits(netuid);

        Self {
            n: Pallet::<T>::get_subnet_n(netuid),
            netuid,
            kappa: Pallet::<T>::get_float_kappa(netuid),

            last_update: Pallet::<T>::get_last_update(netuid),
            block_at_registration: Pallet::<T>::get_last_update(netuid),

            weights: vec![],
            stake: vec![],
            active_stake: vec![],
            preranks: vec![],
            incentive: vec![],

            validator_permits,
            validator_forbids: validator_permits.iter().map(|&b| !b).collect(),
            max_allowed_validators: Pallet::<T>::get_max_allowed_validators(self.netuid),

            _pd: Default::default(),
        }
    }

    fn foo(mut self) {
        let activity_cutoff = 0;

        let current_block: u64 = Pallet::<T>::get_current_block_number();

        let (inactive, active): (Vec<_>, Vec<_>) = self
            .last_update
            .iter()
            .map(|&updated| {
                let is_inactive = updated + activity_cutoff < current_block;
                (is_inactive, !is_inactive)
            })
            .unzip();
        log::trace!("Inactive: {inactive:?}");
        log::trace!("Active: {active:?}");

        self.compute_weights();
        self.compute_stake();
        self.compute_active_stake(&inactive);

        // let new_validator_permits: Vec<bool> = is_topk(&self.stake, self.max_allowed_validators);

        let Consensus {
            consensus,
            validator_trust,
        } = self.compute_consensus();

        self.compute_incentive_and_trust();
    }

    fn compute_weights(&mut self) {
        // Access network weights row unnormalized.
        self.weights = Pallet::<T>::get_weights_sparse(self.netuid);
        // log::trace!("W: {weights:?}");

        // Mask weights that are not from permitted validators.
        self.weights = mask_rows_sparse(&self.validator_forbids, &self.weights);
        // log::trace!("W (permit): {weights:?}");

        // Remove self-weight by masking diagonal.
        self.weights = mask_diag_sparse(&self.weights);
        // log::trace!("W (permit+diag): {weights:?}");

        // Remove weights referring to deregistered neurons.
        self.weights = vec_mask_sparse_matrix(
            &self.weights,
            &self.last_update,
            &self.block_at_registration,
            |updated, registered| updated <= registered,
        );
        // log::trace!("W (permit+diag+outdate): {weights:?}");

        // Normalize remaining weights.
        inplace_row_normalize_sparse(&mut self.weights);
    }

    fn compute_stake(&mut self) {
        let hotkeys: Vec<_> = Keys::<T>::iter_prefix(self.netuid).collect();
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); self.n as usize];
        for (uid_i, hotkey) in &hotkeys {
            // Pallet::<T>::get_total_stake_to(, key)
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        inplace_normalize_64(&mut stake_64);

        self.stake = vec_fixed64_to_fixed32(stake_64); // range: I32F32(0, 1)
                                                       // log::trace!("S: {:?}", &stake);
    }

    fn compute_active_stake(&mut self, inactive: &[bool]) {
        self.active_stake = self.stake.to_vec();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut self.active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&self.validator_forbids, &mut self.active_stake);

        // Normalize active stake.
        inplace_normalize(&mut self.active_stake);
    }

    fn compute_consensus(&self) -> Consensus {
        // Compute preranks: r_j = SUM(i) w_ij * s_i
        self.preranks = matmul_sparse(&self.weights, &self.active_stake, self.n);
        // log::trace!( "R (before): {:?}", &preranks );

        // Clip weights at majority consensus
        let consensus: Vec<I32F32> =
            weighted_median_col_sparse(&self.active_stake, &self.weights, self.n, self.kappa);
        // log::trace!("C: {:?}", &consensus);

        self.weights = col_clip_sparse(&self.weights, &consensus);
        // log::trace!("W: {:?}", &weights);

        let validator_trust: Vec<I32F32> = row_sum_sparse(&self.weights);
        // log::trace!("Tv: {:?}", &validator_trust);

        Consensus {
            consensus,
            validator_trust,
        }
    }

    fn compute_incentive_and_trust(&mut self) {
        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        self.incentive = matmul_sparse(&self.weights, &self.active_stake, self.n);
        // log::trace!("R (after): {:?}", &ranks);

        // Compute server trust: ratio of rank after vs. rank before.
        self.trust = vecdiv(&self.incentive, &self.preranks); // range: I32F32(0, 1)
                                                              // log::trace!("T: {:?}", &trust);

        inplace_normalize(&mut self.incentive); // range: I32F32(0, 1)
                                                // let incentive: Vec<I32F32> = ranks.clone();
                                                // log::trace!("I (=R): {:?}", &incentive);
    }
}

struct Consensus {
    consensus: Vec<I32F32>,
    validator_trust: Vec<I32F32>,
}

impl<T: Config> Pallet<T> {
    fn get_weights_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n = Self::get_subnet_n(netuid) as usize;
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, weights_i) in Weights::<T>::iter_prefix(netuid) {
            if uid_i >= n as u16 {
                continue;
            }

            for (uid_j, weight_ij) in weights_i.iter() {
                if *uid_j >= n as u16 {
                    continue;
                }
                weights[uid_i as usize].push((*uid_j, I32F32::from_num(*weight_ij)));
            }
        }
        weights
    }
}
