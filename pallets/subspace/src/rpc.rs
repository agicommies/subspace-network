use super::*;

// ! Get read structures in a fast way from the blockchain
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct ModuleStats<T: Config> {
    pub last_update: u64,
    pub registration_block: u64,
    pub stake_from: BTreeMap<T::AccountId, u64>, /* map of key to stake on this module/key *
                                                  * (includes delegations) */
    pub emission: u64,
    pub incentive: u16,
    pub dividends: u16,
    pub weights: Vec<(u16, u16)>, // Vec of (uid, weight)
}

impl<T: Config> Pallet<T> {
    pub fn get_module_stats(netuid: u16, key: &T::AccountId) -> ModuleStats<T> {
        let uid = Uids::<T>::get(netuid, key).unwrap_or(u16::MAX);

        let emission = Self::get_emission_for_uid(netuid, uid);
        let incentive = Self::get_incentive_for_uid(netuid, uid);
        let dividends = Self::get_dividends_for_uid(netuid, uid);
        let last_update = Self::get_last_update_for_uid(netuid, uid);

        let weights: Vec<(u16, u16)> = Weights::<T>::get(netuid, uid)
            .iter()
            .filter_map(|(i, w)| if *w > 0 { Some((*i, *w)) } else { None })
            .collect();
        let stake_from: BTreeMap<T::AccountId, u64> = StakeFrom::<T>::get(key);

        let registration_block = RegistrationBlock::<T>::get(netuid, uid);

        ModuleStats {
            stake_from,
            emission,
            incentive,
            dividends,
            last_update,
            registration_block,
            weights,
        }
    }
}
