#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::StaticLookup;

pub use pallet::*;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::too_many_arguments)]

    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::BlockNumberFor;
    pub use sp_std::{vec, vec::Vec};

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config {
        #[pallet::no_default_bounds]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            log::info!(
                "{:?}",
                core::str::from_utf8(&testthing::offworker::decrypt_weight(b"detpyrcne".to_vec()))
            );

            Weight::zero()
        }

        fn offchain_worker(block_number: BlockNumberFor<T>) {
            log::info!("Hello from pallet-ocw.");
            log::info!(
                "{:?}",
                core::str::from_utf8(&testthing::offworker::decrypt_weight(b"detpyrcne".to_vec()))
            );
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_: TransactionSource, _call: &Self::Call) -> TransactionValidity {
            todo!()
        }

        fn pre_dispatch(_: &Self::Call) -> Result<(), TransactionValidityError> {
            todo!()
        }
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}
