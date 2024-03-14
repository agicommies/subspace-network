use super::*;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use substrate_fixed::types::I32F32;

impl<T: Config> Pallet<T> {
    pub fn do_add_profit_shares(
        origin: T::RuntimeOrigin,
        keys: Vec<T::AccountId>,
        shares: Vec<u16>,
    ) -> DispatchResult {
        let key = ensure_signed(origin)?;
        ensure!(
            Self::is_key_registered_on_any_network(&key),
            Error::<T>::NotRegistered
        );
        ensure!(!keys.is_empty(), Error::<T>::EmptyKeys);
        ensure!(keys.len() == shares.len(), Error::<T>::DifferentLengths);

        let total_shares: u32 = shares.iter().map(|x| *x as u32).sum();
        ensure!(total_shares > 0, Error::<T>::InvalidShares);

        let normalized_shares_float: Vec<I32F32> = shares
            .iter()
            .map(|share| {
                (I32F32::from_num(*share) / I32F32::from_num(total_shares as u16))
                    * I32F32::from_num(u16::MAX)
            })
            .collect();

        let normalize_shares: Vec<u16> =
            normalized_shares_float.iter().map(|x| x.to_num::<u16>()).collect();

        let total_normalized_shares: u16 = normalize_shares.iter().sum();
        ensure!(
            total_normalized_shares == u16::MAX,
            Error::<T>::InvalidNormalizedShares
        );

        let profit_share_tuples: Vec<(T::AccountId, u16)> =
            keys.into_iter().zip(normalize_shares.into_iter()).collect();

        ProfitShares::<T>::insert(&key, profit_share_tuples.clone());
        ensure!(
            ProfitShares::<T>::get(&key).len() == profit_share_tuples.len(),
            Error::<T>::ProfitSharesNotAdded
        );

        Ok(())
    }
    pub fn get_profit_share_emissions(
        key: T::AccountId,
        emission: u64,
    ) -> Vec<(T::AccountId, u64)> {
        let profit_shares = ProfitShares::<T>::get(&key);
        profit_shares
            .iter()
            .map(|(share_key, share_ratio)| {
                let share_emission_float: I32F32 = I32F32::from_num(emission)
                    * (I32F32::from_num(*share_ratio) / I32F32::from_num(u16::MAX));
                let share_emission: u64 = share_emission_float.to_num();
                (share_key.clone(), share_emission)
            })
            .collect()
    }

    #[cfg(debug_assertions)]
    pub fn get_profit_shares(key: T::AccountId) -> Vec<(T::AccountId, u16)> {
        ProfitShares::<T>::get(&key)
    }
}
