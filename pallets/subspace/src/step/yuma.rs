use core::marker::PhantomData;

use substrate_fixed::types::{I32F32, I64F64, I96F32};

use crate::{math::*, Config, Kappa, Pallet, Stake, Weights};

pub struct YumaCalc<T: Config> {
    /// The amount of modules on the subnet
    n: u16,
    /// The UID of the subnet
    netuid: u16,
    /// Consensus majority ratio, e.g. 51%.
    kappa: I32F32,

    last_update: Vec<u64>,
    block_at_registration: Vec<u64>,

    weights: Vec<Vec<(u16, I32F32)>>,
    stake: Vec<I32F32>,
    active_stake: Vec<I32F32>,
    preranks: Vec<I32F32>,
    ranks: Vec<I32F32>,
    incentive: Vec<I32F32>,
    dividends: Vec<I32F32>,
    trust: Vec<I32F32>,

    validator_permits: Vec<bool>,
    validator_forbids: Vec<bool>,
    max_allowed_validators: usize,

    _pd: PhantomData<T>,
}

impl<T: Config> YumaCalc<T> {
    pub fn new(netuid: u16) -> Self {
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
            ranks: vec![],
            incentive: vec![],
            dividends: vec![],
            trust: vec![],

            validator_forbids: validator_permits.iter().map(|&b| !b).collect(),
            validator_permits,
            max_allowed_validators: Pallet::<T>::get_max_allowed_validators(netuid),

            _pd: Default::default(),
        }
    }

    pub fn run(mut self) {
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
        self.compute_bonds_and_dividends();
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
        // Access network stake as normalized vector.
        let mut stake: Vec<_> =
            Stake::<T>::iter_prefix_values(self.netuid).map(I64F64::from_num).collect();
        assert_eq!(stake.len(), self.n as usize);

        inplace_normalize_64(&mut stake);

        self.stake = vec_fixed64_to_fixed32(stake); // range: I32F32(0, 1)
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

    fn compute_consensus(&mut self) -> Consensus {
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
        self.ranks = matmul_sparse(&self.weights, &self.active_stake, self.n);
        // log::trace!("R (after): {:?}", &ranks);

        // Compute server trust: ratio of rank after vs. rank before.
        self.trust = vecdiv(&self.ranks, &self.preranks); // range: I32F32(0, 1)
                                                          // log::trace!("T: {:?}", &trust);

        self.incentive = self.ranks.clone();
        inplace_normalize(&mut self.incentive); // range: I32F32(0, 1)
                                                // log::trace!("I (=R): {:?}", &incentive);
    }

    fn compute_bonds_and_dividends(&mut self) -> Bonds {
        // Access network bonds.
        let mut bonds = Pallet::<T>::get_bonds_sparse(self.netuid);
        log::trace!("B: {:?}", &bonds);

        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix(
            &bonds,
            &self.last_update,
            &self.block_at_registration,
            &|updated, registered| updated <= registered,
        );
        log::trace!("B (outdatedmask): {:?}", &bonds);

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse(&mut bonds, self.n);
        log::trace!("B (mask+norm): {:?}", &bonds);

        // Compute bonds delta column normalized.
        let mut bonds_delta = row_hadamard_sparse(&self.weights, &self.active_stake); // ΔB = W◦S (outdated W masked)
        log::trace!("ΔB: {:?}", &bonds_delta);

        // Normalize bonds delta.
        inplace_col_normalize_sparse(&mut bonds_delta, self.n); // sum_i b_ij = 1
        log::trace!("ΔB (norm): {:?}", &bonds_delta);

        // Compute bonds moving average.
        let bonds_moving_average =
            I64F64::from_num(Pallet::<T>::get_bonds_moving_average(self.netuid))
                / I64F64::from_num(1_000_000);
        let alpha = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        let mut ema_bonds = mat_ema_sparse(&bonds_delta, &bonds, alpha);

        // Normalize EMA bonds.
        inplace_col_normalize_sparse(&mut ema_bonds, self.n); // sum_i b_ij = 1
        log::trace!("emaB: {:?}", &ema_bonds);

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends = matmul_transpose_sparse(&ema_bonds, &self.incentive);
        inplace_normalize(&mut dividends);
        log::trace!("D: {:?}", &dividends);

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale_sparse(&mut ema_bonds, self.n);

        Bonds {
            ema_bonds,
            dividends,
        }
    }

    fn compute_emissions(&mut self) {
        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> = self
            .incentive
            .iter()
            .zip(self.dividends.clone())
            .map(|(ii, di)| ii + di)
            .collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = self.incentive.clone(); // Servers get incentive.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        let mut normalized_validator_emission: Vec<I32F32> = self.dividends.clone(); // Validators get dividends.
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&self.active_stake) {
                // no active stake
                normalized_validator_emission = self.stake.clone(); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission = self.stake.clone();
            } else {
                normalized_validator_emission = self.active_stake.clone(); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission = self.active_stake.clone();
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(todo!() as usize);

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
    }
}

struct Consensus {
    consensus: Vec<I32F32>,
    validator_trust: Vec<I32F32>,
}

struct Bonds {
    ema_bonds: Vec<Vec<(u16, I32F32)>>,
    dividends: Vec<I32F32>,
}

struct Emissions {}

impl<T: Config> Pallet<T> {
    pub fn get_float_kappa(netuid: u16) -> I32F32 {
        I32F32::from_num(Kappa::<T>::get(netuid)) / I32F32::from_num(u16::MAX)
    }

    fn get_validator_permits(netuid: u16) -> Vec<bool> {
        todo!()
    }

    fn get_max_allowed_validators(netuid: u16) -> usize {
        todo!()
    }

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

    fn get_bonds_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        todo!()
    }

    fn get_bonds_moving_average(netuid: u16) -> I64F64 {
        todo!()
    }
}
