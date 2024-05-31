use super::*;

use frame_support::{
    traits::{Get, OnRuntimeUpgrade, StorageInstance, StorageVersion},
    weights::Weight,
};

impl<T: Config> StorageInstance for Pallet<T> {
    fn pallet_prefix() -> &'static str {
        "Subspace"
    }

    const STORAGE_PREFIX: &'static str = "Subspace";
}

use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;

pub fn ss58_to_account_id<T: Config>(
    ss58_address: &str,
) -> Result<T::AccountId, sp_core::crypto::PublicError> {
    let account_id = AccountId32::from_ss58check(ss58_address)?;
    let account_id_vec = account_id.encode();
    Ok(T::AccountId::decode(&mut &account_id_vec[..]).unwrap())
}

pub mod v11 {
    use self::{
        global::BurnConfiguration,
        old_storage::{GlobalDaoTreasury, MaxBurn, MinBurn},
    };
    use super::*;

    mod old_storage {
        use super::*;
        use frame_support::{pallet_prelude::ValueQuery, storage_alias};

        #[storage_alias]
        pub type MinBurn<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;

        #[storage_alias]
        pub type MaxBurn<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;

        #[storage_alias]
        pub type AdjustmentAlpha<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;

        #[storage_alias]
        pub type GlobalDaoTreasury<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;
    }

    pub struct MigrateToV11<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV11<T> {
        fn on_runtime_upgrade() -> Weight {
            let on_chain_version = StorageVersion::get::<Pallet<T>>();

            if on_chain_version != 10 {
                log::info!("Storage v11 already updated");
                return Weight::zero();
            }

            let current_adjustment_alpha = old_storage::AdjustmentAlpha::<T>::get();
            // Nuke the old adjustement alpha storage
            old_storage::AdjustmentAlpha::<T>::kill();
            log::info!("Migrating adjustment alpha to v10");

            let mut gaps = BTreeSet::new();
            let netuids: BTreeSet<_> = N::<T>::iter_keys().collect();
            for netuid in 0..netuids.last().copied().unwrap_or_default() {
                // Migrate the adjustment alpha to the new storage
                AdjustmentAlpha::<T>::insert(netuid, current_adjustment_alpha);
                if !netuids.contains(&netuid) {
                    gaps.insert(netuid);
                }
            }

            log::info!("Existing subnets: {netuids:?}");
            log::info!("Updated subnets gaps: {gaps:?}");
            SubnetGaps::<T>::set(gaps);

            let burn_config = BurnConfiguration::<T> {
                min_burn: MinBurn::<T>::get(),
                max_burn: MaxBurn::<T>::get(),
                _pd: PhantomData,
            };

            if let Err(err) = burn_config.apply() {
                log::error!("error migrating burn configurations: {err:?}")
            } else {
                log::info!("Migrated burn-related params to BurnConfig in v10");
            }

            let old_treasury_balance = GlobalDaoTreasury::<T>::get();
            let treasury_account = DaoTreasuryAddress::<T>::get();
            log::info!("Treasury balance: {old_treasury_balance}");
            Pallet::<T>::add_balance_to_account(
                &treasury_account,
                Pallet::<T>::u64_to_balance(old_treasury_balance).unwrap_or_default(),
            );
            GlobalDaoTreasury::<T>::set(0);

            StorageVersion::new(11).put::<Pallet<T>>();
            T::DbWeight::get().writes(1)
        }
    }
}
pub mod v10 {
    use super::*;

    pub struct MigrateToV10<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV10<T> {
        fn on_runtime_upgrade() -> Weight {
            let on_chain_version = StorageVersion::get::<Pallet<T>>();

            if on_chain_version != 9 {
                log::info!("Storage v10 already updated");
                return Weight::zero();
            }

            // Allow more scaling in max_allowed_modules
            MaxAllowedModules::<T>::put(20_000);
            log::info!("Migrated MaxAllowedModules to V10");

            let subnet_0_netuid = 0;
            // Due to the size of the migration, we don't scale below 7k modules
            // Additial modules will be removed by the next migration
            let max_allowed_uids = 7_000; // Current 8k +
                                          // Register modules on s0 that

            let mut total_modules = N::<T>::get(subnet_0_netuid);
            while total_modules > max_allowed_uids {
                let lowest_uid = Pallet::<T>::get_lowest_uid(subnet_0_netuid, false);
                if let Some(uid) = lowest_uid {
                    Pallet::<T>::remove_module(subnet_0_netuid, uid);
                    total_modules -= 1;
                } else {
                    break;
                }
            }
            MaxAllowedUids::<T>::insert(subnet_0_netuid, max_allowed_uids);
            log::info!("Migrated Modules on subnet 0 to V10");

            log::info!(
                "Module amount on subnet 0 is {:?}",
                N::<T>::get(subnet_0_netuid)
            );

            StorageVersion::new(10).put::<Pallet<T>>();
            log::info!("Migrated Registration Intervals to V9");

            let mut gaps = BTreeSet::new();
            let netuids: BTreeSet<_> = N::<T>::iter_keys().collect();
            for netuid in 0..netuids.last().copied().unwrap_or_default() {
                if !netuids.contains(&netuid) {
                    gaps.insert(netuid);
                }
            }

            log::info!("Existing subnets: {netuids:?}");
            log::info!("Updated subnets gaps: {gaps:?}");
            SubnetGaps::<T>::set(gaps);

            T::DbWeight::get().writes(1)
        }
    }
}
