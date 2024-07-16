use super::*;

use frame_support::traits::{Get, StorageInstance, StorageVersion};

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

/// ! First migration running in global stake update
pub mod v12 {
    use super::*;
    use dispatch::DispatchResult;
    use frame_support::{storage::with_storage_layer, traits::OnRuntimeUpgrade, weights::Weight};
    use module::ModuleChangeset;
    use pallet_governance_api::VoteMode;
    use pallet_subnet_emission_api::SubnetConsensus;
    use sp_runtime::Percent;
    use sp_std::collections::btree_map::BTreeMap;

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

        #[storage_alias]
        pub type TotalStake<T: Config> = StorageMap<Pallet<T>, Identity, u16, u64, ValueQuery>;

        #[storage_alias]
        pub type UnitEmission<T: Config> = StorageValue<Pallet<T>, u64, ValueQuery>;

        #[storage_alias]
        pub type PendingEmission<T: Config> = StorageMap<Pallet<T>, Identity, u16, u64, ValueQuery>;

        #[storage_alias]
        pub type SubnetEmission<T: Config> = StorageMap<Pallet<T>, Identity, u16, u64, ValueQuery>;

        #[storage_alias]
        pub type MinStake<T: Config> = StorageMap<Pallet<T>, Identity, u16, u64>;
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

            // --- 1 collect and cleanup the old storages

            // --- 1.1 Nuke the old `Stake` storage, we will no longer support this StorageValue
            let _ = old_storage::Stake::<T>::clear(u32::MAX, None);
            let _ = old_storage::MinStake::<T>::clear(u32::MAX, None);

            // --- 1.2 Download existing data into separate types
            let old_stake_from = old_storage::StakeFrom::<T>::iter().fold(
                BTreeMap::<AccountIdOf<T>, BTreeMap<AccountIdOf<T>, u64>>::new(),
                |mut acc, (_, key, stake)| {
                    let existing = acc.entry(key).or_default();
                    for (key, stake) in stake {
                        existing
                            .entry(key)
                            .and_modify(|existing| *existing = existing.saturating_add(stake))
                            .or_insert(stake);
                    }
                    acc
                },
            );
            let old_stake_to = old_storage::StakeTo::<T>::iter().fold(
                BTreeMap::<AccountIdOf<T>, BTreeMap<AccountIdOf<T>, u64>>::new(),
                |mut acc, (_, key, stake)| {
                    let existing = acc.entry(key).or_default();
                    for (key, stake) in stake {
                        existing
                            .entry(key)
                            .and_modify(|existing| *existing = existing.saturating_add(stake))
                            .or_insert(stake);
                    }
                    acc
                },
            );

            // --- 1. 3 Before migration counts
            let stake_from_count_before: usize =
                old_stake_from.values().map(|stakes| stakes.len()).sum();
            let stake_to_count_before: usize =
                old_stake_to.values().map(|stakes| stakes.len()).sum();

            // --- 1.4 Nuke the old `StakeFrom` and `StakeTo` storages, they will be initialized
            // again from the downloaded data
            let _ = StakeTo::<T>::clear(u32::MAX, None);
            let _ = StakeFrom::<T>::clear(u32::MAX, None);

            // --- 2. Initialize the Stake storages again

            // --- 2. 1  Migrate StakeFrom
            for (from, stakes) in old_stake_from {
                for (to, amount) in stakes {
                    StakeFrom::<T>::insert(&from, &to, amount);
                }
            }
            log::info!("Migrated StakeFrom");

            // --- 2.2  Migrate StakeTo
            for (to, stakes) in old_stake_to {
                for (from, amount) in stakes {
                    StakeTo::<T>::insert(&to, &from, amount);
                }
            }
            log::info!("Migrated StakeTo");

            // --- 3. Migrate TotalStake

            // --- 3.1 Download and sum
            let total_stake: u64 =
                old_storage::TotalStake::<T>::iter().map(|(_, stake)| stake).sum();

            // --- 3.2 Nuke the old `TotalStake` storage
            let _ = old_storage::TotalStake::<T>::clear(u32::MAX, None);

            // --- 3.3 Reinitialize the TotalStake storage again
            TotalStake::<T>::set(total_stake);
            log::info!("Migrated TotalStake");

            // --- 4. Log if everything went as expected
            let stake_from_count_after = StakeFrom::<T>::iter().count();
            let stake_to_count_after = StakeTo::<T>::iter().count();
            // Log results
            let log_result = |name: &str, before: usize, after: usize| {
                if before == after {
                    log::info!("{} count: {} (unchanged)", name, before);
                } else {
                    log::warn!("{} count: {} before, {} after", name, before, after);
                }
            };

            log_result("StakeFrom", stake_from_count_before, stake_from_count_after);
            log_result("StakeTo", stake_to_count_before, stake_to_count_after);

            let stake_from_sum: u64 = StakeFrom::<T>::iter_values().sum();
            let stake_to_sum: u64 = StakeTo::<T>::iter_values().sum();

            log::info!("Total stake is now: {:?}", TotalStake::<T>::get());
            log::info!("Stake from is now: {:?}", stake_from_sum);
            log::info!("Stake to is now: {:?}", stake_to_sum);

            log::info!("-------------------");
            log::info!("STAKE MIGRATION DONE");
            log::info!("-------------------");
            // --- 5. Done for stake storage

            // ========================
            // Subnets migration TL;DR
            // ========================
            /*
            ====================
            Currently
            ====================
            Storages are structured like this
            NETUID     | NAME
            - Subnet 0 | Linear
            - Subnet 1 | Zangief subnet
            - Subnet 2 | Comchat subnet

            ====================
            After migration
            ====================

            NETUID     | NAME
            - Subnet 0 | Rootnet
            - Subnet 1 | Treasury subnet
            - Subnet 2 | Linear Subnet

            ------------------------------------------------------------

            Question is, what do we do with subnet 1,2

            Determine the free netuid values:

            let netuid = netuid.unwrap_or_else(|| match SubnetGaps::<T>::get().first().copied() {
                Some(removed) => removed,
                None => TotalSubnets::<T>::get(),
            });

            and move the subnet 1 + 2, to these free netuid spots.

            When you have done this, move subnet 0 to subnet 2

            And insert the new rootnet into SN0, and SN1 for treasury subnet.

            ------------------------------------------------------------

            Both new subnets, Root & Treasury, have to be registered with specific parameters:

            -------------------------

            - Rootnet:
            set_max_allowed_uids to the number of allowed rootnet validators.
            set vote mode to vote
            set founder fee to 0
            set subnet consensus type to Root

            -------------------------
            - Treasury subnet:
            set_founder the treasury account
            set_founder_fee to 100
            set_vote_mode to vote
            set max_allowed_uids to 0
            set subnet consensus type to Treasury

             -------------------------

            - For linear don't change any parameters, just set
            the consensus type to Linear.

            */

            // --- 1 Deregister "dead" subnets
            // Deregister subnets that don't even have the SubnetEmission storage (inactive)
            // They will need to go through the burned register, instead of having "free" innactive
            // spot.
            for (netuid, emission) in old_storage::SubnetEmission::<T>::iter() {
                if emission == 0 {
                    log::info!("removing subnnets with no emission, netuid {:?}", netuid);
                    Pallet::<T>::remove_subnet(netuid)
                }
            }

            if let Err(err) = with_storage_layer(|| {
                // --- 2 Migrate the subnets

                // --- 2.1 Transfer subnet 1,2 to free spots
                transfer_subnet::<T>(1, None)?;
                transfer_subnet::<T>(2, None)?;
                // --- 2.2 Transfer subnet 0 to subnet 2
                transfer_subnet::<T>(0, Some(2))?;

                // --- 2.3 Overwrite the subnets for Rootnet, Treasury, Linear

                // Rootnet configuration
                // This will be netuid 0
                const ROOTNET_ID: u16 = 0;
                start_subnet::<T>(
                    ROOTNET_ID,
                    "Rootnet",
                    T::get_dao_treasury_address(),
                    SubnetConsensus::Root,
                );
                MaxAllowedUids::<T>::set(ROOTNET_ID, 256);
                MaxAllowedValidators::<T>::set(ROOTNET_ID, Some(256));
                set_vote_mode::<T>(ROOTNET_ID);
                FounderShare::<T>::set(ROOTNET_ID, 0);
                Burn::<T>::set(ROOTNET_ID, 0);
                Pallet::<T>::append_module(
                    ROOTNET_ID,
                    &T::get_dao_treasury_address(),
                    ModuleChangeset::new(
                        b"system".to_vec(),
                        b"system".to_vec(),
                        Percent::from_parts(100),
                        None,
                    ),
                )?;

                // Treasury subnet configuration
                // This will be netuid 1
                const TREASURYNET_ID: u16 = 1;
                start_subnet::<T>(
                    TREASURYNET_ID,
                    "Treasury",
                    T::get_dao_treasury_address(),
                    SubnetConsensus::Treasury,
                );
                set_vote_mode::<T>(TREASURYNET_ID);
                FounderShare::<T>::set(TREASURYNET_ID, u16::MAX);
                MaxAllowedUids::<T>::set(TREASURYNET_ID, 0);
                Pallet::<T>::append_module(
                    TREASURYNET_ID,
                    &T::get_dao_treasury_address(),
                    ModuleChangeset::new(
                        b"system".to_vec(),
                        b"system".to_vec(),
                        Percent::from_parts(100),
                        None,
                    ),
                )?;

                // Linear subnet configuration
                // This will be netuid 2
                const LINEARNET_ID: u16 = 2;
                T::set_subnet_consensus_type(LINEARNET_ID, Some(SubnetConsensus::Linear));

                // --- 3. Initialize the MinImmunityStake storage for all subnets.

                // Migrate freshly created subnet parameter MIN_IMMUNITY_STAKE, to all existing
                // subnets
                let base_min_immunity_stake = 50_000_000_000_000; // 50k
                N::<T>::iter_keys().for_each(|netuid| {
                    // New parameter
                    MinImmunityStake::<T>::insert(netuid, base_min_immunity_stake);
                });

                log::info!("===============================");
                log::info!("MIGRATED SUBNETS");
                log::info!("===============================");
                Ok(()) as DispatchResult
            }) {
                log::error!("could not complete the rootnet migration: {err:?}");
            };

            // --- 4. Finally update global parameters / rootnet config

            // Set kappa to 37k
            Kappa::<T>::put(37_000);
            // Set rho to 12
            Rho::<T>::put(12);
            log::info!("migrated global rootnet consensus variables.");

            // --- 6. Done
            log::info!("==Storage v12 migration done for Subspace Pallet==");

            // --- 6.1 Update the storage version
            StorageVersion::new(12).put::<Pallet<T>>();
            T::DbWeight::get().reads_writes(1, 1)
        }
    }

    fn set_vote_mode<T: Config>(subnet_id: u16) {
        let mut rootnet_governance_configuration =
            T::get_subnet_governance_configuration(subnet_id);
        rootnet_governance_configuration.vote_mode = VoteMode::Vote;

        if let Err(err) =
            T::update_subnet_governance_configuration(subnet_id, rootnet_governance_configuration)
        {
            log::error!(
                "could not update ROOTNET governance configuration: {:?}",
                err
            );
        };
    }

    fn start_subnet<T: Config>(
        subnet_id: u16,
        subnet_name: &'static str,
        founder: T::AccountId,
        consensus_type: SubnetConsensus,
    ) {
        // Bonds
        BondsMovingAverage::<T>::set(subnet_id, 900_000);
        ValidatorPermits::<T>::set(subnet_id, Vec::new());
        ValidatorTrust::<T>::set(subnet_id, Vec::new());
        PruningScores::<T>::set(subnet_id, Vec::new());
        MaxAllowedValidators::<T>::set(subnet_id, None);
        Consensus::<T>::set(subnet_id, Vec::new());
        Active::<T>::set(subnet_id, Vec::new());
        Rank::<T>::set(subnet_id, Vec::new());
        RegistrationsThisInterval::<T>::set(subnet_id, 0);
        Burn::<T>::set(subnet_id, 0);
        MaximumSetWeightCallsPerEpoch::<T>::set(subnet_id, None);
        TargetRegistrationsInterval::<T>::set(subnet_id, 142);
        TargetRegistrationsPerInterval::<T>::set(subnet_id, 3);
        AdjustmentAlpha::<T>::set(subnet_id, u64::MAX / 2);
        MinImmunityStake::<T>::set(subnet_id, 50_000_000_000_000);
        SubnetNames::<T>::set(subnet_id, subnet_name.as_bytes().to_vec());
        N::<T>::insert(subnet_id, 0);
        Founder::<T>::set(subnet_id, founder);
        IncentiveRatio::<T>::set(subnet_id, 50);
        MaxAllowedUids::<T>::set(subnet_id, 420);
        ImmunityPeriod::<T>::set(subnet_id, 0);
        MinAllowedWeights::<T>::set(subnet_id, 1);
        // MaxRegistrationsPerInterval::<T>::set(subnet_id, T::DefaultMaxRegistrationsPerInterval);
        MaxWeightAge::<T>::set(subnet_id, 3600);
        MaxAllowedWeights::<T>::set(subnet_id, 420);
        TrustRatio::<T>::set(subnet_id, 0);
        Tempo::<T>::set(subnet_id, 100);
        FounderShare::<T>::set(subnet_id, 8);
        // Uids
        // Keys
        // Name
        // Address
        // Metadata
        Incentive::<T>::set(subnet_id, Vec::new());
        Trust::<T>::set(subnet_id, Vec::new());
        Dividends::<T>::set(subnet_id, Vec::new());
        Emission::<T>::set(subnet_id, Vec::new());
        LastUpdate::<T>::set(subnet_id, Vec::new());
        // RegistrationBlock
        // Weights
        // DelegationFee
        // DelegationFee
        old_storage::PendingEmission::<T>::set(subnet_id, 0);
        old_storage::SubnetEmission::<T>::set(subnet_id, 0);
        T::set_subnet_consensus_type(subnet_id, Some(consensus_type));
    }

    fn transfer_subnet<T: Config>(curr: u16, target: Option<u16>) -> DispatchResult {
        let target =
            target.unwrap_or_else(|| match SubnetGaps::<T>::mutate(|set| set.pop_first()) {
                Some(removed) => removed,
                None => {
                    let id = TotalSubnets::<T>::get();
                    TotalSubnets::<T>::mutate(|value| *value = value.saturating_add(1));
                    id
                }
            });

        log::info!("transferring subnet {} to {}", curr, target);

        macro_rules! migrate_double_map {
            ($map:ident) => {
                let keys = $map::<T>::iter_key_prefix(&curr).collect::<Vec<_>>();
                for k2 in keys {
                    $map::<T>::swap(&curr, &k2, &target, &k2);
                }
            };
        }

        macro_rules! migrate_map {
            ($map:ident) => {
                $map::<T>::swap(curr, target);
            };
        }

        macro_rules! migrate_api {
            ($getter:ident, $setter:ident) => {
                let curr_value = T::$getter(curr);
                let target_value = T::$getter(target);
                T::$setter(curr, target_value);
                T::$setter(target, curr_value);
            };
        }

        macro_rules! migrate_storage_alias {
            ($storage:ty) => {
                let curr_value = <$storage>::get(curr);
                let target_value = <$storage>::get(target);
                <$storage>::insert(curr, target_value);
                <$storage>::insert(target, curr_value);
            };
        }

        migrate_double_map!(Bonds);
        migrate_map!(BondsMovingAverage);
        migrate_map!(ValidatorPermits);
        migrate_map!(ValidatorTrust);
        migrate_map!(PruningScores);
        migrate_map!(MaxAllowedValidators);
        migrate_map!(Consensus);
        migrate_map!(Active);
        migrate_map!(Rank);
        migrate_map!(RegistrationsThisInterval);
        migrate_map!(Burn);
        migrate_map!(MaximumSetWeightCallsPerEpoch);
        migrate_double_map!(SetWeightCallsPerEpoch);
        migrate_map!(TargetRegistrationsInterval);
        migrate_map!(TargetRegistrationsPerInterval);
        migrate_map!(AdjustmentAlpha);
        migrate_map!(MinImmunityStake);
        migrate_map!(SubnetNames);
        migrate_map!(N);
        migrate_map!(Founder);
        migrate_map!(IncentiveRatio);
        migrate_map!(MaxAllowedUids);
        migrate_map!(ImmunityPeriod);
        migrate_map!(MinAllowedWeights);
        migrate_map!(MaxRegistrationsPerInterval);
        migrate_map!(MaxWeightAge);
        migrate_map!(MaxAllowedWeights);
        migrate_map!(TrustRatio);
        migrate_map!(Tempo);
        migrate_map!(FounderShare);
        migrate_double_map!(Uids);
        migrate_double_map!(Keys);
        migrate_double_map!(Name);
        migrate_double_map!(Address);
        migrate_double_map!(Metadata);
        migrate_map!(Incentive);
        migrate_map!(Trust);
        migrate_map!(Dividends);
        migrate_map!(Emission);
        migrate_map!(LastUpdate);
        migrate_double_map!(RegistrationBlock);
        migrate_double_map!(Weights);
        migrate_double_map!(DelegationFee);
        // Pending emission
        migrate_storage_alias!(old_storage::PendingEmission<T>);
        // Subnet emission
        migrate_storage_alias!(old_storage::SubnetEmission<T>);
        migrate_api!(get_subnet_consensus_type, set_subnet_consensus_type);
        migrate_map!(SubnetRegistrationBlock);

        let curr_governance_config = T::get_subnet_governance_configuration(curr);
        let target_governance_config = T::get_subnet_governance_configuration(curr);
        T::update_subnet_governance_configuration(curr, target_governance_config)?;
        T::update_subnet_governance_configuration(target, curr_governance_config)?;

        Ok(())
    }
}
