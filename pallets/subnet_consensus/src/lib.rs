pub mod linear;
pub use pallet::*;
pub mod math;
pub mod yuma;

#[derive(Debug)]
#[allow(dead_code)]
pub enum EmissionError {
    EmittedMoreThanExpected { emitted: u64, expected: u64 },
    HasEmissionRemaining { emitted: u64 },
    Other(&'static str),
}

impl From<&'static str> for EmissionError {
    fn from(v: &'static str) -> Self {
        Self::Other(v)
    }
}

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::too_many_arguments)]

    use substrate_fixed::types::I64F64;

    use frame_support::{pallet_prelude::*, traits::Currency};
    use pallet_subspace::{DelegationFee, FloorDelegationFee, FounderShare, ProfitShares};
    use sp_runtime::Percent;

    use crate::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config + pallet_subspace::Config {
        /// The events emitted on proposal changes.
        #[pallet::no_default_bounds]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Currency type that will be used to place deposits on modules
        type Currency: Currency<Self::AccountId> + Send + Sync;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    impl<T: Config> Pallet<T> {
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

        pub fn get_profit_share_emissions(
            key: &T::AccountId,
            emission: u64,
        ) -> Vec<(T::AccountId, u64)> {
            let profit_shares = ProfitShares::<T>::get(key);

            profit_shares
                .into_iter()
                .map(|(share_key, share_ratio)| {
                    let share_emission = emission * share_ratio as u64 / u16::MAX as u64;
                    (share_key, share_emission)
                })
                .collect()
        }

        // Returns the delegation fee of a module
        pub fn get_delegation_fee(netuid: u16, module_key: &T::AccountId) -> Percent {
            let min_deleg_fee_global = FloorDelegationFee::<T>::get();
            let delegation_fee = DelegationFee::<T>::get(netuid, module_key);

            delegation_fee.max(min_deleg_fee_global)
        }

        pub fn get_ownership_ratios(
            netuid: u16,
            module_key: &T::AccountId,
        ) -> Vec<(T::AccountId, I64F64)> {
            let stake_from_vector = pallet_subspace::Pallet::<T>::get_stake_from_vector(module_key);
            let _uid = pallet_subspace::Pallet::<T>::get_uid_for_key(netuid, module_key);
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
    }
}
