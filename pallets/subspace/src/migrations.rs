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

pub mod v12 {
    use super::*;
    use frame_support::storage::with_storage_layer;
    use sp_runtime::DispatchError;

    pub mod old_storage {

        use super::*;
        use frame_support::{pallet_prelude::ValueQuery, storage_alias, Identity};
        use sp_std::collections::btree_map::BTreeMap;

        #[storage_alias]
        pub type Stake<T: Config> =
            StorageDoubleMap<Pallet<T>, Identity, u16, Identity, AccountIdOf<T>, u64, ValueQuery>;

        #[storage_alias]
        pub type StakeFrom<T: Config> = StorageDoubleMap<
            Pallet<T>,
            Identity,
            u16,
            Identity,
            AccountIdOf<T>,
            BTreeMap<AccountIdOf<T>, u64>,
            ValueQuery,
        >;

        #[storage_alias]
        pub type StakeTo<T: Config> = StorageDoubleMap<
            Pallet<T>,
            Identity,
            u16,
            Identity,
            AccountIdOf<T>,
            BTreeMap<AccountIdOf<T>, u64>,
            ValueQuery,
        >;
    }

    pub struct MigrateToV12<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrateToV12<T> {
        fn on_runtime_upgrade() -> Weight {
            let on_chain_version = StorageVersion::get::<Pallet<T>>();

            if on_chain_version != 11 {
                log::info!("Storage v12 already updated");
                return Weight::zero();
            }
            log::info!("Migrating storage to v12");

            if let Err(err) = with_storage_layer(|| {
                for (_, b, c) in old_storage::Stake::<T>::iter() {
                    let current_stake = Stake::<T>::get(&b);
                    Stake::<T>::set(
                        b,
                        current_stake.checked_add(c).ok_or(Error::<T>::ArithmeticError)?,
                    );
                }
                let _ = old_storage::Stake::<T>::clear(u32::MAX, None);

                for (_, b, c) in old_storage::StakeTo::<T>::iter() {
                    for (key, stake) in c {
                        let current = StakeTo::<T>::get(&b, &key);
                        StakeTo::<T>::set(
                            &b,
                            &key,
                            current.checked_add(stake).ok_or(Error::<T>::ArithmeticError)?,
                        );
                    }
                }
                let _ = old_storage::StakeTo::<T>::clear(u32::MAX, None);

                for (_, b, c) in old_storage::StakeFrom::<T>::iter() {
                    for (key, stake) in c {
                        let current = StakeFrom::<T>::get(&b, &key);
                        StakeFrom::<T>::set(
                            &b,
                            &key,
                            current.checked_add(stake).ok_or(Error::<T>::ArithmeticError)?,
                        );
                    }
                }
                let _ = old_storage::StakeFrom::<T>::clear(u32::MAX, None);

                Ok::<(), DispatchError>(())
            }) {
                log::error!("could not migrate stake related storage values: {err:?}");
            };

            StorageVersion::new(12).put::<Pallet<T>>();
            T::DbWeight::get().writes(1)
        }
    }

    // TODO:
    //
}
