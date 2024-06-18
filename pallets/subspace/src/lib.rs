#![allow(deprecated, non_camel_case_types, non_snake_case)]
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]

use crate::subnet::SubnetChangeset;
use frame_system::{self as system, ensure_signed};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
// export the migrations here
pub mod migrations;

use frame_support::{
    dispatch,
    dispatch::{DispatchInfo, PostDispatchInfo},
    ensure,
    storage::with_storage_layer,
    traits::{tokens::WithdrawReasons, Currency, ExistenceRequirement, IsSubType},
    PalletId,
};

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::Weight, sp_runtime::transaction_validity::ValidTransaction};
use sp_runtime::{
    traits::{
        AccountIdConversion, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension,
    },
    transaction_validity::{TransactionValidity, TransactionValidityError},
};
use sp_std::marker::PhantomData;

// ---------------------------------
//	Benchmark Imports
// ---------------------------------

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// ---------------------------------
// Pallet Imports
// ---------------------------------

// This is needed so other pallets can acess
#[allow(unused_imports)]
pub use pallet::*;
pub mod global;
pub mod module;
mod profit_share;
mod registration;
pub mod rpc;
mod set_weights;
mod staking;
pub mod subnet;
pub mod subnet_consensus;
pub mod voting;
pub mod weights; // Weight benchmarks // Commune consensus weights

// TODO: better error handling in whole file

#[frame_support::pallet]
pub mod pallet {
    #![allow(
        deprecated,
        clippy::let_unit_value,
        clippy::too_many_arguments,
        clippy::type_complexity
    )]

    use self::voting::{CuratorApplication, Proposal, VoteMode};
    pub use crate::weights::WeightInfo;

    use super::*;
    use frame_support::{pallet_prelude::*, traits::Currency, Identity};
    use frame_system::pallet_prelude::*;

    use global::BurnConfiguration;
    use module::ModuleChangeset;
    use sp_arithmetic::per_things::Percent;
    use sp_runtime::DispatchResult;
    pub use sp_std::{vec, vec::Vec};

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(8);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config {
        /// This pallet's ID, used for generating the treasury account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        #[pallet::no_default_bounds]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // --- Currency type that will be used to place deposits on modules
        type Currency: Currency<Self::AccountId> + Send + Sync;

        // The weight information of this pallet.
        type WeightInfo: WeightInfo;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    // ---------------------------------
    // Global Variables
    // ---------------------------------

    #[pallet::type_value]
    pub fn DefaultBurnConfig<T: Config>() -> BurnConfiguration<T> {
        BurnConfiguration {
            min_burn: 4_000_000_000,
            max_burn: 250_000_000_000,
            adjustment_alpha: u64::MAX / 2,
            adjustment_interval: DefaultTempo::<T>::get() * 2,
            expected_registrations: DefaultTempo::<T>::get(),
            _pd: PhantomData,
        }
    }

    #[pallet::storage]
    pub type BurnConfig<T: Config> =
        StorageValue<_, BurnConfiguration<T>, ValueQuery, DefaultBurnConfig<T>>;

    #[pallet::type_value]
    pub fn DefaultSubnetBurnConfig<T: Config>() -> BurnConfiguration<T> {
        BurnConfiguration {
            min_burn: 2_000_000_000_000,
            max_burn: 100_000_000_000_000,
            adjustment_alpha: (DefaultBurnConfig::<T>::get().adjustment_alpha as f32 * 1.2) as u64,
            adjustment_interval: 2_000,
            expected_registrations: 1,
            _pd: PhantomData,
        }
    }

    #[pallet::storage]
    pub type SubnetBurnConfig<T: Config> =
        StorageValue<_, BurnConfiguration<T>, ValueQuery, DefaultSubnetBurnConfig<T>>;

    #[pallet::type_value]
    pub fn DefaultAdjustmentAlpha<T: Config>() -> u64 {
        u64::MAX / 2
    }

    #[pallet::type_value]
    pub fn DefaultKappa<T: Config>() -> u16 {
        32_767 // This coresponds to 0,5 (majority of stake agreement)
    }

    #[pallet::storage]
    pub type Kappa<T> = StorageValue<_, u16, ValueQuery, DefaultKappa<T>>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> bonds
    pub type Bonds<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultBondsMovingAverage<T: Config>() -> u64 {
        900_000
    }

    #[pallet::storage] // --- MAP ( netuid ) --> bonds_moving_average
    pub type BondsMovingAverage<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultBondsMovingAverage<T>>;

    #[pallet::storage] // --- DMAP ( netuid ) --> validator_permit
    pub type ValidatorPermits<T: Config> = StorageMap<_, Identity, u16, Vec<bool>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> validator_trust
    pub type ValidatorTrust<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> pruning_scores
    pub type PruningScores<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMaxAllowedValidators<T: Config>() -> Option<u16> {
        None // Some(128)
    }

    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_validators
    pub type MaxAllowedValidators<T> =
        StorageMap<_, Identity, u16, Option<u16>, ValueQuery, DefaultMaxAllowedValidators<T>>;

    #[pallet::storage] // --- DMAP ( netuid ) --> consensus
    pub type Consensus<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> active
    pub type Active<T: Config> = StorageMap<_, Identity, u16, Vec<bool>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid ) --> rank
    pub type Rank<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMaxNameLength<T: Config>() -> u16 {
        32
    }
    #[pallet::storage] // --- ITEM ( max_name_length )
    pub type MaxNameLength<T: Config> = StorageValue<_, u16, ValueQuery, DefaultMaxNameLength<T>>;

    #[pallet::type_value]
    pub fn DefaultMinNameLength<T: Config>() -> u16 {
        2
    }

    #[pallet::storage]
    pub type MinNameLength<T: Config> = StorageValue<_, u16, ValueQuery, DefaultMinNameLength<T>>;

    #[pallet::type_value]
    pub fn DefaultMaxAllowedSubnets<T: Config>() -> u16 {
        256
    }
    #[pallet::storage] // --- ITEM ( max_allowed_subnets )
    pub type MaxAllowedSubnets<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultMaxAllowedSubnets<T>>;

    #[pallet::storage]
    // --- MAP (netuid) --> registrations_this_interval
    pub(super) type RegistrationsThisInterval<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;

    #[pallet::storage]
    pub type SubnetRegistrationsThisInterval<T: Config> = StorageValue<_, u16, ValueQuery>;

    #[pallet::storage]
    // --- MAP (netuid) --> burn
    pub type Burn<T: Config> = StorageMap<_, Identity, u16, u64, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultSubnetBurn<T: Config>() -> u64 {
        SubnetBurnConfig::<T>::get().min_burn
    }

    #[pallet::storage]
    pub type SubnetBurn<T: Config> = StorageValue<_, u64, ValueQuery, DefaultSubnetBurn<T>>;

    #[pallet::type_value]
    pub fn DefaultMaxAllowedModules<T: Config>() -> u16 {
        10_000
    }
    #[pallet::storage] // --- ITEM ( max_allowed_modules )
    pub type MaxAllowedModules<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultMaxAllowedModules<T>>;

    #[pallet::storage] // --- ITEM ( registrations_this block )
    pub type RegistrationsPerBlock<T> = StorageValue<_, u16, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMaxRegistrationsPerBlock<T: Config>() -> u16 {
        10
    }
    #[pallet::storage] // --- ITEM( global_max_registrations_per_block )
    pub type MaxRegistrationsPerBlock<T> =
        StorageValue<_, u16, ValueQuery, DefaultMaxRegistrationsPerBlock<T>>;

    #[pallet::type_value]
    pub fn DefaultMinDelegationFeeGlobal<T: Config>() -> Percent {
        Percent::from_percent(5u8)
    }

    #[pallet::storage]
    pub type FloorDelegationFee<T> =
        StorageValue<_, Percent, ValueQuery, DefaultMinDelegationFeeGlobal<T>>;

    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MinWeightStake<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMaxAllowedWeightsGlobal<T: Config>() -> u16 {
        512
    }
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MaxAllowedWeightsGlobal<T> =
        StorageValue<_, u16, ValueQuery, DefaultMaxAllowedWeightsGlobal<T>>;

    #[pallet::storage]
    pub type MaximumSetWeightCallsPerEpoch<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery>;

    #[pallet::storage]
    pub type SetWeightCallsPerEpoch<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, T::AccountId, u16, ValueQuery>;

    #[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct ModuleParams<T: Config> {
        pub name: Vec<u8>,
        pub address: Vec<u8>,
        pub delegation_fee: Percent,
        pub metadata: Option<Vec<u8>>,
        pub controller: T::AccountId,
    }

    #[derive(
        Decode, Encode, PartialEq, Eq, Clone, TypeInfo, frame_support::DebugNoBound, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct GlobalParams<T: Config> {
        // max
        pub max_name_length: u16,             // max length of a network name
        pub min_name_length: u16,             // min length of a network name
        pub max_allowed_subnets: u16,         // max number of subnets allowed
        pub max_allowed_modules: u16,         // max number of modules allowed per subnet
        pub max_registrations_per_block: u16, // max number of registrations per block
        pub max_allowed_weights: u16,         // max number of weights per module

        // mins
        pub floor_delegation_fee: Percent, // min delegation fee
        pub floor_founder_share: u8,       // min founder share
        pub min_weight_stake: u64,         // min weight stake required

        // proposals
        pub proposal_cost: u64,
        pub proposal_expiration: u32,
        pub proposal_participation_threshold: Percent,

        // S0 governance
        pub curator: T::AccountId,
        pub general_subnet_application_cost: u64,

        // Other
        pub burn_config: BurnConfiguration<T>,
    }

    // ---------------------------------
    // Subnet PARAMS
    // ---------------------------------

    pub struct DefaultSubnetParams<T: Config>(sp_std::marker::PhantomData<((), T)>);

    impl<T: Config> DefaultSubnetParams<T> {
        pub fn get() -> SubnetParams<T> {
            SubnetParams {
                name: BoundedVec::default(),
                tempo: DefaultTempo::<T>::get(),
                immunity_period: DefaultImmunityPeriod::<T>::get(),
                min_allowed_weights: DefaultMinAllowedWeights::<T>::get(),
                max_allowed_weights: DefaultMaxAllowedWeights::<T>::get(),
                max_allowed_uids: DefaultMaxAllowedUids::<T>::get(),
                max_weight_age: DefaultMaxWeightAge::<T>::get(),
                trust_ratio: GetDefault::get(),
                founder_share: FloorFounderShare::<T>::get() as u16,
                incentive_ratio: DefaultIncentiveRatio::<T>::get(),
                min_stake: 0,
                founder: DefaultKey::<T>::get(),
                vote_mode: DefaultVoteMode::<T>::get(),
                maximum_set_weight_calls_per_epoch: 0,
                bonds_ma: DefaultBondsMovingAverage::<T>::get(),
            }
        }
    }

    #[derive(
        Decode, Encode, PartialEq, Eq, Clone, frame_support::DebugNoBound, TypeInfo, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct SubnetParams<T: Config> {
        // --- parameters
        pub founder: T::AccountId,
        pub founder_share: u16,   // out of 100
        pub immunity_period: u16, // immunity period
        pub incentive_ratio: u16, // out of 100
        pub max_allowed_uids: u16, /* max number of weights allowed to be registered in this
                                   * pub max_allowed_uids: u16, // max number of uids
                                   * allowed to be registered in this subne */
        pub max_allowed_weights: u16, /* max number of weights allowed to be registered in this
                                       * pub max_allowed_uids: u16, // max number of uids
                                       * allowed to be registered in this subnet */
        pub min_allowed_weights: u16, // min number of weights allowed to be registered in this
        pub max_weight_age: u64,      // max age of a weight
        pub min_stake: u64,           // min stake required
        pub name: BoundedVec<u8, ConstU32<256>>,
        pub tempo: u16, // how many blocks to wait before rewarding models
        pub trust_ratio: u16,
        pub maximum_set_weight_calls_per_epoch: u16,
        pub vote_mode: VoteMode,
        // consensus
        pub bonds_ma: u64,
    }

    #[pallet::type_value]
    pub fn DefaultMaxAllowedUids<T: Config>() -> u16 {
        820
    }
    #[pallet::storage] // --- MAP ( netuid ) --> max_allowed_uids
    pub type MaxAllowedUids<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedUids<T>>;

    #[pallet::type_value]
    pub fn DefaultImmunityPeriod<T: Config>() -> u16 {
        40
    }
    #[pallet::storage] // --- MAP ( netuid ) --> immunity_period
    pub type ImmunityPeriod<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultImmunityPeriod<T>>;

    #[pallet::type_value]
    pub fn DefaultMinAllowedWeights<T: Config>() -> u16 {
        1
    }
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MinAllowedWeights<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMinAllowedWeights<T>>;

    #[pallet::type_value]
    pub fn DefaultSelfVote<T: Config>() -> bool {
        false
    }
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type SelfVote<T> = StorageMap<_, Identity, u16, bool, ValueQuery, DefaultSelfVote<T>>;

    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MinStake<T> = StorageMap<_, Identity, u16, u64, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultMaxWeightAge<T: Config>() -> u64 {
        3600 // 3.6k blocks, that is 8 hours
    }
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MaxWeightAge<T> =
        StorageMap<_, Identity, u16, u64, ValueQuery, DefaultMaxWeightAge<T>>;

    #[pallet::type_value]
    pub fn DefaultMaxAllowedWeights<T: Config>() -> u16 {
        420
    }
    #[pallet::storage] // --- MAP ( netuid ) --> min_allowed_weights
    pub type MaxAllowedWeights<T> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultMaxAllowedWeights<T>>;

    #[pallet::storage] // --- DMAP ( key, netuid ) --> bool
    pub type Founder<T: Config> =
        StorageMap<_, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T>>;

    #[pallet::storage] // --- DMAP ( key, netuid ) --> bool
    pub type FounderShare<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultFounderShare<T>>;

    #[pallet::type_value]
    pub fn DefaultFounderShare<T: Config>() -> u16 {
        FloorFounderShare::<T>::get() as u16
    }

    #[pallet::type_value]
    pub fn DefaultIncentiveRatio<T: Config>() -> u16 {
        50
    }
    #[pallet::storage] // --- DMAP ( key, netuid ) --> bool
    pub type IncentiveRatio<T: Config> =
        StorageMap<_, Identity, u16, u16, ValueQuery, DefaultIncentiveRatio<T>>;

    #[pallet::type_value]
    pub fn DefaultTempo<T: Config>() -> u16 {
        100
    }
    #[pallet::storage] // --- MAP ( netuid ) --> epoch
    pub type Tempo<T> = StorageMap<_, Identity, u16, u16, ValueQuery, DefaultTempo<T>>;

    #[pallet::storage] // --- MAP ( netuid ) --> epoch
    pub type TrustRatio<T> = StorageMap<_, Identity, u16, u16, ValueQuery>;

    // ---------------------------------
    // Voting
    // ---------------------------------

    #[pallet::type_value]
    pub fn DefaultCurator<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
    }

    #[pallet::storage]
    pub type FloorFounderShare<T: Config> =
        StorageValue<_, u8, ValueQuery, DefaultFloorFounderShare<T>>;

    #[pallet::type_value] // This has to be different than DefaultKey, so we are not conflicting in tests.
    pub fn DefaultDaoTreasuryAddress<T: Config>() -> T::AccountId {
        T::PalletId::get().into_account_truncating()
    }

    #[pallet::storage]
    pub type DaoTreasuryAddress<T: Config> =
        StorageValue<_, T::AccountId, ValueQuery, DefaultDaoTreasuryAddress<T>>;

    #[pallet::type_value]
    pub fn DefaultFloorFounderShare<T: Config>() -> u8 {
        8
    }

    #[pallet::storage]
    pub type Curator<T: Config> = StorageValue<_, T::AccountId, ValueQuery, DefaultKey<T>>;

    #[pallet::type_value]
    pub fn DefaultVoteMode<T: Config>() -> VoteMode {
        VoteMode::Authority
    }

    #[pallet::storage] // --- MAP ( netuid ) --> epoch
    pub type VoteModeSubnet<T> =
        StorageMap<_, Identity, u16, VoteMode, ValueQuery, DefaultVoteMode<T>>;

    #[pallet::storage] // --- ITEM( tota_number_of_existing_networks )
    pub type TotalSubnets<T> = StorageValue<_, u16, ValueQuery>;

    #[pallet::storage] // --- MAP ( netuid ) --> subnetwork_n (Number of UIDs in the network).
    pub type N<T> = StorageMap<_, Identity, u16, u16, ValueQuery>;

    #[pallet::storage] // --- MAP ( network_name ) --> netuid
    pub type SubnetNames<T: Config> = StorageMap<_, Identity, u16, Vec<u8>, ValueQuery>;

    // ---------------------------------
    // Module Variables
    // ---------------------------------

    #[pallet::storage] // --- DMAP ( netuid, module_key ) --> uid
    pub type Uids<T: Config> =
        StorageDoubleMap<_, Identity, u16, Blake2_128Concat, T::AccountId, u16, OptionQuery>;

    #[pallet::type_value]
    pub fn DefaultKey<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes()).unwrap()
    }
    #[pallet::storage] // --- DMAP ( netuid, uid ) --> module_key
    pub type Keys<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, T::AccountId, ValueQuery, DefaultKey<T>>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> module_name
    pub type Name<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, u16, Vec<u8>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> module_address
    pub type Address<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, u16, Vec<u8>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid, module key ) --> metadata_uri
    pub type Metadata<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u16, Twox64Concat, T::AccountId, Vec<u8>>;

    #[pallet::type_value]
    pub fn DefaultDelegationFee<T: Config>() -> Percent {
        Percent::from_percent(20u8)
    }
    #[pallet::storage] // -- DMAP(netuid, module_key) -> delegation_fee
    pub type DelegationFee<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u16,
        Blake2_128Concat,
        T::AccountId,
        Percent,
        ValueQuery,
        DefaultDelegationFee<T>,
    >;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> block number that the module is registered
    pub type RegistrationBlock<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, u64, ValueQuery>;

    //  Module Staking Variables
    /// ========================

    #[pallet::storage]
    pub type Stake<T: Config> = StorageMap<_, Identity, T::AccountId, u64, ValueQuery>;

    // TODO: luiz, conver this to double maps
    #[pallet::storage]
    pub type StakeFrom<T: Config> =
        StorageMap<_, Identity, T::AccountId, BTreeMap<T::AccountId, u64>, ValueQuery>;

    #[pallet::storage]
    pub type StakeTo<T: Config> =
        StorageMap<_, Identity, T::AccountId, BTreeMap<T::AccountId, u64>, ValueQuery>;

    // #[pallet::storage]
    // pub type StakeTo<T: Config> = StorageDoubleMap<
    //     _,
    //     Identity,
    //     T::AccountId,
    //     Identity,
    //     T::AccountId,
    //     u64,
    //     ValueQuery,
    // >;

    // Subnets
    // =======

    #[pallet::storage] // --- MAP( netuid ) --> lowest_subnet
    pub type SubnetGaps<T> = StorageValue<_, BTreeSet<u16>, ValueQuery>;

    #[pallet::storage]
    pub type SubnetEmission<T> = StorageMap<_, Identity, u16, u64, ValueQuery>;

    #[pallet::storage]
    pub type TotalStake<T> = StorageValue<_, u64, ValueQuery>;

    // PROFIT SHARE VARIABLES
    #[pallet::storage] // --- DMAP ( netuid, account_id ) --> Vec<(module_key, stake )> | Returns the list of the
    pub type ProfitShares<T: Config> =
        StorageMap<_, Identity, T::AccountId, Vec<(T::AccountId, u16)>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultProfitShareUnit<T: Config>() -> u16 {
        u16::MAX
    }
    #[pallet::storage]
    pub type ProfitShareUnit<T: Config> =
        StorageValue<_, u16, ValueQuery, DefaultProfitShareUnit<T>>;

    // ---------------------------------
    // Module Consensus Variables
    // ---------------------------------

    #[pallet::storage] // --- MAP ( netuid ) --> incentive
    pub type Incentive<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> trust
    pub type Trust<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> dividends
    pub type Dividends<T: Config> = StorageMap<_, Identity, u16, Vec<u16>, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> emission
    pub type Emission<T: Config> = StorageMap<_, Identity, u16, Vec<u64>, ValueQuery>;
    #[pallet::storage] // --- MAP ( netuid ) --> last_update
    pub type LastUpdate<T: Config> = StorageMap<_, Identity, u16, Vec<u64>, ValueQuery>;

    #[pallet::storage] // --- DMAP ( netuid, uid ) --> weights
    pub type Weights<T: Config> =
        StorageDoubleMap<_, Identity, u16, Identity, u16, Vec<(u16, u16)>, ValueQuery>;

    // whitelist for the base subnet (netuid 0)
    #[pallet::storage]
    pub type LegitWhitelist<T: Config> = StorageMap<_, Identity, T::AccountId, u8, ValueQuery>;

    // ---------------------------------
    // Event Variables
    // ---------------------------------

    // TODO:
    // emit all events that are not being emitted !
    #[pallet::event]
    #[pallet::generate_deposit(pub fn deposit_event)]
    pub enum Event<T: Config> {
        NetworkAdded(u16, Vec<u8>), // --- Event created when a new network is added.
        NetworkRemoved(u16),        // --- Event created when a network is removed.
        StakeAdded(T::AccountId, T::AccountId, u64), /* --- Event created when stake has been
                                     * transfered from the a coldkey account
                                     * onto the key staking account. */
        StakeRemoved(T::AccountId, T::AccountId, u64), /* --- Event created when stake has been
                                                        * removed from the key staking account
                                                        * onto the coldkey account. */
        WeightsSet(u16, u16), /* ---- Event created when a caller successfully sets their
                               * weights on a subnetwork. */
        ModuleRegistered(u16, u16, T::AccountId), /* --- Event created when a new module
                                                   * account has been registered to the chain. */
        ModuleDeregistered(u16, u16, T::AccountId), /* --- Event created when a module account
                                                     * has been deregistered from the chain. */
        WhitelistModuleAdded(T::AccountId), /* --- Event created when a module account has been
                                             * added to the whitelist. */
        WhitelistModuleRemoved(T::AccountId), /* --- Event created when a module account has
                                               * been removed from the whitelist. */
        BulkModulesRegistered(u16, u16), /* --- Event created when multiple uids have been
                                          * concurrently registered. */
        BulkBalancesSet(u16, u16),
        MaxAllowedUidsSet(u16, u16), /* --- Event created when max allowed uids has been set
                                      * for a subnetwor. */
        MinAllowedWeightSet(u16, u16), /* --- Event created when minimun allowed weight is set
                                        * for a subnet. */
        ImmunityPeriodSet(u16, u16), /* --- Event created when immunity period is set for a
                                      * subnet. */
        ModuleUpdated(u16, T::AccountId), /* --- Event created when the module server
                                           * information is added to the network. */
        MaxNameLengthSet(u16), // --- Event created when setting the maximum network name length
        MinNameLenghtSet(u16), // --- Event created when setting the minimum network name length
        MaxAllowedSubnetsSet(u16), // --- Event created when setting the maximum allowed subnets
        MaxAllowedModulesSet(u16), // --- Event created when setting the maximum allowed modules
        MaxRegistrationsPerBlockSet(u16), // --- Event created when we set max registrations
        target_registrations_intervalSet(u16), // --- Event created when we set target registrations
        RegistrationBurnChanged(u64),

        // faucet
        Faucet(T::AccountId, BalanceOf<T>), // (id, balance_to_add)

        //voting
        ProposalCreated(u64),                        // id of the proposal
        ApplicationCreated(u64),                     // id of the application
        ProposalVoted(u64, T::AccountId, bool),      // (id, voter, vote)
        ProposalVoteUnregistered(u64, T::AccountId), // (id, voter)
        GlobalParamsUpdated(GlobalParams<T>),        /* --- Event created when global
                                                      * parameters are
                                                      * updated */
        SubnetParamsUpdated(u16), // --- Event created when subnet parameters are updated
        GlobalProposalAccepted(u64), // (id)
        CustomProposalAccepted(u64), // (id)
        SubnetProposalAccepted(u64, u16), // (id, netuid)
    }

    // ---------------------------------
    // Error Variables
    // ---------------------------------

    // TODO:
    // comment all error variables
    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NetworkDoesNotExist, // --- Thrown when the network does not exist.
        NoSlotAvailable,     // --- Thrown when subnet is full.
        TooFewVotesForNewProposal,
        NetworkExist, // --- Thrown when the network already exist.
        InvalidIpType, /* ---- Thrown when the user tries to serve an module which
                       * is not of type	4 (IPv4) or 6 (IPv6). */
        NotRegistered, // module which does not exist in the active set.
        NotEnoughStakeToWithdraw, /* ---- Thrown when the caller requests removing more stake
                        * then there exists in the staking account. See: fn
                        * remove_stake. */
        NotEnoughBalanceToStake, /*  ---- Thrown when the caller requests adding more stake
                                  * than there exists in the cold key account. See: fn
                                  * add_stake */
        BalanceWithdrawalError, /* ---- Thrown when the caller tries to add stake, but for some
                                 * reason the requested amount could not be withdrawn from the
                                 * coldkey account */
        WeightVecNotEqualSize, /* ---- Thrown when the caller attempts to set the weight keys
                                * and values but these vectors have different size. */
        DuplicateUids, /* ---- Thrown when the caller attempts to set weights with duplicate
                        * uids in the weight matrix. */
        InvalidUid, /* ---- Thrown when a caller attempts to set weight to at least one uid
                     * that does not exist in the metagraph. */
        InvalidUidsLength, /* ---- Thrown when the caller attempts to set weights with a
                            * different number of uids than allowed. */
        NotSettingEnoughWeights, /* ---- Thrown when the dispatch attempts to set weights on
                                  * chain with fewer elements than are allowed. */
        TooManyRegistrationsPerBlock, /* ---- Thrown when registrations this block exceeds
                                       * allowed number. */
        AlreadyRegistered, /* ---- Thrown when the caller requests registering a module which
                            * already exists in the active set. */
        MaxAllowedUIdsNotAllowed, // ---  Thrown if the vaule is invalid for MaxAllowedUids
        CouldNotConvertToBalance, /* ---- Thrown when the dispatch attempts to convert between
                                   * a u64 and T::balance but the call fails. */
        StakeAlreadyAdded, /* --- Thrown when the caller requests adding stake for a key to the
                            * total stake which already added */
        StorageValueOutOfRange, /* --- Thrown when the caller attempts to set a storage value
                                 * outside of its allowed range. */
        TempoHasNotSet, // --- Thrown when epoch has not set
        InvalidTempo,   // --- Thrown when epoch is not valid
        SettingWeightsTooFast, /* --- Thrown if the key attempts to set weights twice withing
                         * net_epoch/2 blocks. */
        BalanceSetError, // --- Thrown when an error occurs setting a balance
        MaxAllowedUidsExceeded, /* --- Thrown when number of accounts going to be registered
                          * exceed MaxAllowedUids for the network. */
        TooManyUids, /* ---- Thrown when the caller attempts to set weights with more uids than
                      * are allowed. */
        InvalidMaxAllowedUids, /* --- Thrown when the user tries to set max allowed uids to a
                                * value less than the current number of registered uids. */
        NetuidDoesNotExist,
        SubnetNameAlreadyExists,
        MissingSubnetName,
        SubnetNameTooShort,
        SubnetNameTooLong,
        InvalidSubnetName,
        BalanceNotAdded,
        StakeNotRemoved,
        KeyAlreadyRegistered,
        EmptyKeys,
        TooManyKeys,
        NotCurator, /* --- Thrown when the user tries to set the curator and is not the
                     * curator */
        ApplicationNotFound,
        AlreadyWhitelisted, /* --- Thrown when the user tries to whitelist an account that is
                             * already whitelisted. */
        NotWhitelisted, /* --- Thrown when the user tries to remove an account from the
                         * whitelist that is not whitelisted. */
        InvalidShares,
        ProfitSharesNotAdded,
        NotFounder,
        NameAlreadyRegistered,
        NotEnoughStakeToSetWeights,
        NotEnoughStakeToStartNetwork,
        NotEnoughStakePerWeight,
        NoSelfWeight,
        DifferentLengths,
        NotEnoughBalanceToRegister,
        NotEnoughBalanceToRegisterSubnet,
        StakeNotAdded,
        BalanceNotRemoved,
        BalanceCouldNotBeRemoved,
        NotEnoughStakeToRegister,
        StillRegistered,
        MaxAllowedModules, /* --- Thrown when the user tries to set max allowed modules to a
                            * value less than the current number of registered modules. */
        NotEnoughBalanceToTransfer,
        NotVoteMode,
        InvalidTrustRatio,
        InvalidMinAllowedWeights,
        InvalidMaxAllowedWeights,
        InvalidMinStake,
        InvalidMinDelegationFee,
        InvalidModuleMetadata,
        ModuleMetadataTooLong,

        InvalidMaxNameLength,
        InvalidMinNameLenght,
        InvalidMaxAllowedSubnets,
        InvalidMaxAllowedModules,
        InvalidMaxRegistrationsPerBlock,
        InvalidTargetRegistrationsInterval,
        InvalidVoteThreshold,
        InvalidUnitEmission,
        InvalidMinBurn,
        InvalidMaxBurn,
        InvalidTargetRegistrationsPerInterval,

        // Faucet
        FaucetDisabled, // --- Thrown when the faucet is disabled.
        InvalidDifficulty,
        InvalidWorkBlock,
        InvalidSeal,
        InvalidBalance,

        // Modules
        /// The module name is too long.
        ModuleNameTooLong,
        ModuleNameTooShort,
        /// The module name is invalid. It has to be a UTF-8 encoded string.
        InvalidModuleName,
        /// The address is too long.
        ModuleAddressTooLong,
        /// The module address is invalid.
        InvalidModuleAddress,
        /// The module name does not exist in the subnet.
        ModuleNameDoesNotExist,
        /// A module with this name already exists in the subnet.
        ModuleNameAlreadyExists,
        /// A module with this name already exists in the subnet.
        // VOTING
        ProposalNotFound,
        InvalidProposalStatus,
        InvalidProposalData,
        AlreadyVoted,
        InvalidVoteMode,
        InvalidImmunityPeriod,
        InvalidFounderShare,
        InvalidIncentiveRatio,

        InvalidProposalCost,
        InvalidGeneralSubnetApplicationCost,
        InvalidProposalExpiration,
        InvalidProposalParticipationThreshold,
        InsufficientStake,
        VoteNotFound,
        InvalidProposalCustomData,
        ProposalCustomDataTooSmall,
        ProposalCustomDataTooLarge,
        // DAO / Governance
        ApplicationTooSmall,
        ApplicationTooLarge,
        ApplicationNotPending,
        InvalidApplication,
        NotEnoughBalanceToPropose,
        NotEnoughtBalnceToApply,

        // Other
        InvalidMaxWeightAge,
        InvalidRecommendedWeight,
        ArithmeticError,

        MaximumSetWeightsPerEpochReached,
        InsufficientDaoTreasuryFunds,
    }

    // ---------------------------------
    // Genesis
    // ---------------------------------

    #[derive(frame_support::DefaultNoBound)]
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        // key, name, address, weights
        pub modules: Vec<Vec<(T::AccountId, Vec<u8>, Vec<u8>, Vec<(u16, u16)>)>>,
        // name, tempo, immunity_period, min_allowed_weight, max_allowed_weight, max_allowed_uids,
        // immunity_ratio, founder
        pub subnets: Vec<(Vec<u8>, u16, u16, u16, u16, u16, u64, T::AccountId)>,

        pub stake_to: Vec<Vec<(T::AccountId, Vec<(T::AccountId, u64)>)>>,

        pub block: u32,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial total issuance from balances
            // Subnet config values

            for (subnet_idx, subnet) in self.subnets.iter().enumerate() {
                let netuid: u16 = subnet_idx as u16;
                // --- Set subnet parameters

                let params: SubnetParams<T> = SubnetParams {
                    name: subnet.0.clone().try_into().expect("subnet name is too long"),
                    tempo: subnet.1,
                    immunity_period: subnet.2,
                    min_allowed_weights: subnet.3,
                    max_allowed_weights: subnet.4,
                    max_allowed_uids: subnet.5,
                    min_stake: subnet.6,
                    founder: subnet.7.clone(),
                    ..DefaultSubnetParams::<T>::get()
                };

                let fee = DelegationFee::<T>::get(netuid, &params.founder);
                let changeset: SubnetChangeset<T> =
                    SubnetChangeset::new(params).expect("genesis subnets are valid");
                let _ = self::Pallet::<T>::add_subnet(changeset, Some(netuid))
                    .expect("Failed to register genesis subnet");
                for (uid_usize, (key, name, address, weights)) in
                    self.modules[subnet_idx].iter().enumerate()
                {
                    let changeset = ModuleChangeset::new(name.clone(), address.clone(), fee, None);
                    self::Pallet::<T>::append_module(netuid, key, changeset)
                        .expect("genesis modules are valid");
                    Weights::<T>::insert(netuid, uid_usize as u16, weights);
                }
            }
            // Now we can add the stake to the network
            for (subnet_idx, _subnet) in self.subnets.iter().enumerate() {
                let netuid: u16 = subnet_idx as u16;

                for (key, stake_to) in self.stake_to[netuid as usize].iter() {
                    for (module_key, stake_amount) in stake_to {
                        self::Pallet::<T>::increase_stake(key, module_key, *stake_amount);
                    }
                }
            }
        }
    }

    // ---------------------------------
    // Proposals
    // ---------------------------------

    // Global Parameters of proposals

    // TODO:
    // move majority to the governance pallet
    #[pallet::type_value]
    pub fn DefaultProposalCost<T: Config>() -> u64 {
        10_000_000_000_000 // 10_000 $COMAI, the value is returned if the proosal passes
    }

    #[pallet::storage]
    pub type ProposalCost<T: Config> = StorageValue<_, u64, ValueQuery, DefaultProposalCost<T>>;

    #[pallet::type_value]
    pub fn DefaultProposalExpiration<T: Config>() -> u32 {
        130000 // Aprox 12 days
    }

    #[pallet::storage]
    pub type ProposalExpiration<T: Config> =
        StorageValue<_, u32, ValueQuery, DefaultProposalExpiration<T>>;

    #[pallet::type_value]
    pub fn DefaultProposalParticipationThreshold<T: Config>() -> Percent {
        Percent::from_percent(50)
    }

    #[pallet::storage]
    pub(super) type ProposalParticipationThreshold<T: Config> =
        StorageValue<_, Percent, ValueQuery, DefaultProposalParticipationThreshold<T>>;

    #[pallet::type_value]
    pub fn DefaultGeneralSubnetApplicationCost<T: Config>() -> u64 {
        1_000_000_000_000 // 1_000 $COMAI
    }

    #[pallet::storage]
    pub type GeneralSubnetApplicationCost<T: Config> =
        StorageValue<_, u64, ValueQuery, DefaultGeneralSubnetApplicationCost<T>>;

    #[pallet::storage]
    pub type Proposals<T: Config> = StorageMap<_, Identity, u64, Proposal<T>>;

    #[pallet::storage]
    pub type CuratorApplications<T: Config> = StorageMap<_, Identity, u64, CuratorApplication<T>>;

    // ---------------------------------
    // Hooks
    // ---------------------------------

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// ---- Called on the initialization of this pallet. (the order of on_finalize calls is
        /// determined in the runtime)
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            let block_number: u64 =
                block_number.try_into().ok().expect("blockchain won't pass 2 ^ 64 blocks");

            // Make sure to use storage layer, so no panic in initialization hook can't happen
            let res: DispatchResult = with_storage_layer(|| {
                Self::adjust_registration_parameters(block_number)?;
                Self::adjust_subnet_registration_parameters(block_number)?;
                Ok(())
            });

            if let Err(e) = res {
                log::error!("Error in on_initialize: {:?}", e);
            }
            Weight::zero()
        }
    }

    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.

    // ---------------------------------
    // Extrinsics
    // ---------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ---------------------------------
        // Consensus operations
        // ---------------------------------

        #[pallet::call_index(0)]
        #[pallet::weight((T::WeightInfo::set_weights(), DispatchClass::Normal, Pays::No))]
        pub fn set_weights(
            origin: OriginFor<T>,
            netuid: u16,
            uids: Vec<u16>,
            weights: Vec<u16>,
        ) -> DispatchResult {
            Self::do_set_weights(origin, netuid, uids, weights)
        }

        // ---------------------------------
        // Stake operations
        // ---------------------------------

        #[pallet::call_index(1)]
        #[pallet::weight((T::WeightInfo::add_stake(), DispatchClass::Normal, Pays::No))]
        pub fn add_stake(
            origin: OriginFor<T>,
            module_key: T::AccountId,
            amount: u64,
        ) -> DispatchResult {
            Self::do_add_stake(origin, module_key, amount)
        }

        #[pallet::call_index(2)]
        #[pallet::weight((T::WeightInfo::remove_stake(), DispatchClass::Normal, Pays::No))]
        pub fn remove_stake(
            origin: OriginFor<T>,
            module_key: T::AccountId,
            amount: u64,
        ) -> DispatchResult {
            Self::do_remove_stake(origin, module_key, amount)
        }

        // ---------------------------------
        // Bulk stake operations
        // ---------------------------------

        #[pallet::call_index(3)]
        #[pallet::weight((T::WeightInfo::add_stake_multiple(), DispatchClass::Normal, Pays::No))]
        pub fn add_stake_multiple(
            origin: OriginFor<T>,
            module_keys: Vec<T::AccountId>,
            amounts: Vec<u64>,
        ) -> DispatchResult {
            Self::do_add_stake_multiple(origin, module_keys, amounts)
        }

        #[pallet::call_index(4)]
        #[pallet::weight((T::WeightInfo::remove_stake_multiple(), DispatchClass::Normal, Pays::No))]
        pub fn remove_stake_multiple(
            origin: OriginFor<T>,
            module_keys: Vec<T::AccountId>,
            amounts: Vec<u64>,
        ) -> DispatchResult {
            Self::do_remove_stake_multiple(origin, module_keys, amounts)
        }

        // ---------------------------------
        // Transfers
        // ---------------------------------

        #[pallet::call_index(5)]
        #[pallet::weight((T::WeightInfo::transfer_stake(), DispatchClass::Normal, Pays::No))]
        pub fn transfer_stake(
            origin: OriginFor<T>,         // --- The account that is calling this function.
            module_key: T::AccountId,     // --- The module key.
            new_module_key: T::AccountId, // --- The new module key.
            amount: u64,                  // --- The amount of stake to transfer.
        ) -> DispatchResult {
            Self::do_transfer_stake(origin, module_key, new_module_key, amount)
        }

        #[pallet::call_index(6)]
        #[pallet::weight((T::WeightInfo::transfer_multiple(), DispatchClass::Normal, Pays::No))]
        pub fn transfer_multiple(
            origin: OriginFor<T>, // --- The account that is calling this function.
            destinations: Vec<T::AccountId>, // --- The module key.
            amounts: Vec<u64>,    // --- The amount of stake to transfer.
        ) -> DispatchResult {
            Self::do_transfer_multiple(origin, destinations, amounts)
        }

        // ---------------------------------
        // Registereing / Deregistering
        // ---------------------------------

        #[pallet::call_index(7)]
        #[pallet::weight((T::WeightInfo::register(), DispatchClass::Normal, Pays::No))]
        pub fn register(
            origin: OriginFor<T>,
            network: Vec<u8>,
            name: Vec<u8>,
            address: Vec<u8>,
            stake: u64,
            module_key: T::AccountId,
            metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            Self::do_register(origin, network, name, address, stake, module_key, metadata)
        }

        #[pallet::call_index(8)]
        #[pallet::weight((T::WeightInfo::deregister(), DispatchClass::Normal, Pays::No))]
        pub fn deregister(origin: OriginFor<T>, netuid: u16) -> DispatchResult {
            Self::do_deregister(origin, netuid)
        }

        // ---------------------------------
        // Updating
        // ---------------------------------

        #[pallet::call_index(9)]
        #[pallet::weight((T::WeightInfo::deregister(), DispatchClass::Normal, Pays::No))]
        pub fn update_module(
            origin: OriginFor<T>,
            netuid: u16,
            name: Vec<u8>,
            address: Vec<u8>,
            delegation_fee: Option<Percent>,
            metadata: Option<Vec<u8>>,
        ) -> DispatchResult {
            let key = ensure_signed(origin.clone())?;
            ensure!(
                Self::is_registered(Some(netuid), &key),
                Error::<T>::NotRegistered
            );

            let params = Self::module_params(netuid, &key);

            let changeset =
                ModuleChangeset::update(&params, name, address, delegation_fee, metadata);
            Self::do_update_module(origin, netuid, changeset)
        }

        #[pallet::call_index(10)]
        #[pallet::weight((T::WeightInfo::update_subnet(), DispatchClass::Normal, Pays::No))]
        pub fn update_subnet(
            origin: OriginFor<T>,
            netuid: u16,
            founder: T::AccountId,
            founder_share: u16,
            immunity_period: u16,
            incentive_ratio: u16,
            max_allowed_uids: u16,
            max_allowed_weights: u16,
            min_allowed_weights: u16,
            max_weight_age: u64,
            min_stake: u64,
            name: BoundedVec<u8, ConstU32<256>>,
            tempo: u16,
            trust_ratio: u16,
            maximum_set_weight_calls_per_epoch: u16,
            vote_mode: VoteMode,
            bonds_ma: u64,
        ) -> DispatchResult {
            let params = SubnetParams {
                founder,
                founder_share,
                immunity_period,
                incentive_ratio,
                max_allowed_uids,
                max_allowed_weights,
                min_allowed_weights,
                max_weight_age,
                min_stake,
                name,
                tempo,
                trust_ratio,
                maximum_set_weight_calls_per_epoch,
                vote_mode,
                bonds_ma,
            };

            let changeset = SubnetChangeset::update(netuid, params)?;
            Self::do_update_subnet(origin, netuid, changeset)
        }

        // ---------------------------------
        // Subnet 0 DAO
        // ---------------------------------

        #[pallet::call_index(11)]
        #[pallet::weight((T::WeightInfo::add_dao_application(), DispatchClass::Normal, Pays::No))]
        pub fn add_dao_application(
            origin: OriginFor<T>,
            application_key: T::AccountId,
            data: Vec<u8>,
        ) -> DispatchResult {
            Self::do_add_dao_application(origin, application_key, data)
        }

        #[pallet::call_index(12)]
        #[pallet::weight((T::WeightInfo::refuse_dao_application(), DispatchClass::Normal, Pays::No))]
        pub fn refuse_dao_application(origin: OriginFor<T>, id: u64) -> DispatchResult {
            Self::do_refuse_dao_application(origin, id)
        }

        #[pallet::call_index(13)]
        #[pallet::weight((T::WeightInfo::add_to_whitelist(), DispatchClass::Normal, Pays::No))]
        pub fn add_to_whitelist(
            origin: OriginFor<T>,
            module_key: T::AccountId,
            recommended_weight: u8,
        ) -> DispatchResult {
            Self::do_add_to_whitelist(origin, module_key, recommended_weight)
        }

        #[pallet::call_index(14)]
        #[pallet::weight((T::WeightInfo::remove_from_whitelist(), DispatchClass::Normal, Pays::No))]
        pub fn remove_from_whitelist(
            origin: OriginFor<T>,
            module_key: T::AccountId,
        ) -> DispatchResult {
            Self::do_remove_from_whitelist(origin, module_key)
        }

        // ---------------------------------
        // Adding proposals
        // ---------------------------------

        #[pallet::call_index(15)]
        #[pallet::weight((T::WeightInfo::add_global_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn add_global_proposal(
            origin: OriginFor<T>,
            max_name_length: u16,             // max length of a network name
            min_name_length: u16,             // min length of a network name
            max_allowed_subnets: u16,         // max number of subnets allowed
            max_allowed_modules: u16,         // max number of modules allowed per subnet
            max_registrations_per_block: u16, // max number of registrations per block
            max_allowed_weights: u16,         // max number of weights per module
            max_burn: u64,                    // max burn allowed to register
            min_burn: u64,                    // min burn required to register
            floor_delegation_fee: Percent,    // min delegation fee
            floor_founder_share: u8,          // min founder share
            min_weight_stake: u64,            // min weight stake required
            target_registrations_per_interval: u16, /* desired number of registrations per
                                               * interval */
            target_registrations_interval: u16, /* the number of blocks that defines the
                                                 * registration interval */
            adjustment_alpha: u64, // adjustment alpha
            curator: T::AccountId, // subnet 0 dao multisig
            proposal_cost: u64,    /*amount of $COMAI to create a proposal
                                    * returned if proposal gets accepted */
            proposal_expiration: u32, // the block number, proposal expires at
            proposal_participation_threshold: Percent, /*  minimum stake of the overall network
                                       * stake,
                                       *  in order for proposal to get executed */
            general_subnet_application_cost: u64,
        ) -> DispatchResult {
            let mut params = Self::global_params();
            params.max_name_length = max_name_length;
            params.min_name_length = min_name_length;
            params.max_allowed_subnets = max_allowed_subnets;
            params.max_allowed_modules = max_allowed_modules;
            params.max_registrations_per_block = max_registrations_per_block;
            params.max_allowed_weights = max_allowed_weights;
            params.floor_delegation_fee = floor_delegation_fee;
            params.floor_founder_share = floor_founder_share;
            params.min_weight_stake = min_weight_stake;
            params.curator = curator;
            params.proposal_cost = proposal_cost;
            params.proposal_expiration = proposal_expiration;
            params.proposal_participation_threshold = proposal_participation_threshold;
            params.general_subnet_application_cost = general_subnet_application_cost;

            params.burn_config.min_burn = min_burn;
            params.burn_config.max_burn = max_burn;
            params.burn_config.adjustment_alpha = adjustment_alpha;
            params.burn_config.adjustment_interval = target_registrations_interval;
            params.burn_config.expected_registrations = target_registrations_per_interval;

            Self::do_add_global_proposal(origin, params)
        }

        #[pallet::call_index(16)]
        #[pallet::weight((T::WeightInfo::add_subnet_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn add_subnet_proposal(
            origin: OriginFor<T>,
            netuid: u16,
            founder: T::AccountId,
            name: BoundedVec<u8, ConstU32<256>>,
            founder_share: u16, // out of 100
            immunity_period: u16,
            incentive_ratio: u16, // out of 100
            max_allowed_uids: u16, /* max number of weights allowed to be
                                   * registered in this
                                   * subnet */
            max_allowed_weights: u16, /* max number of weights allowed to be registered in this
                                       * subnet */
            min_allowed_weights: u16, /* min number of weights allowed to be registered in this
                                       * subnet */
            min_stake: u64,
            max_weight_age: u64,
            tempo: u16, // how many blocks to wait before rewarding models
            trust_ratio: u16,
            maximum_set_weight_calls_per_epoch: u16,
            vote_mode: VoteMode,
            bonds_ma: u64,
        ) -> DispatchResult {
            let mut params = Self::subnet_params(netuid);
            params.founder = founder;
            params.name = name;
            params.founder_share = founder_share;
            params.immunity_period = immunity_period;
            params.incentive_ratio = incentive_ratio;
            params.max_allowed_uids = max_allowed_uids;
            params.max_allowed_weights = max_allowed_weights;
            params.min_allowed_weights = min_allowed_weights;
            params.min_stake = min_stake;
            params.max_weight_age = max_weight_age;
            params.tempo = tempo;
            params.trust_ratio = trust_ratio;
            params.maximum_set_weight_calls_per_epoch = maximum_set_weight_calls_per_epoch;
            params.vote_mode = vote_mode;
            params.bonds_ma = bonds_ma;
            Self::do_add_subnet_proposal(origin, netuid, params)
        }

        #[pallet::call_index(17)]
        #[pallet::weight((T::WeightInfo::add_custom_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn add_custom_proposal(origin: OriginFor<T>, data: Vec<u8>) -> DispatchResult {
            Self::do_add_custom_proposal(origin, data)
        }

        #[pallet::call_index(18)]
        #[pallet::weight((T::WeightInfo::add_custom_subnet_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn add_custom_subnet_proposal(
            origin: OriginFor<T>,
            netuid: u16,
            data: Vec<u8>,
        ) -> DispatchResult {
            Self::do_add_custom_subnet_proposal(origin, netuid, data)
        }

        #[pallet::call_index(19)]
        #[pallet::weight((T::WeightInfo::add_transfer_dao_treasury_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn add_transfer_dao_treasury_proposal(
            origin: OriginFor<T>,
            data: Vec<u8>,
            value: u64,
            dest: T::AccountId,
        ) -> DispatchResult {
            Self::do_add_transfer_dao_treasury_proposal(origin, data, value, dest)
        }

        // ---------------------------------
        // Voting / Unvoting proposals
        // ---------------------------------

        #[pallet::call_index(20)]
        #[pallet::weight((T::WeightInfo::vote_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn vote_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
            agree: bool,
        ) -> DispatchResult {
            Self::do_vote_proposal(origin, proposal_id, agree)
        }

        #[pallet::call_index(21)]
        #[pallet::weight((T::WeightInfo::unvote_proposal(), DispatchClass::Normal, Pays::No))]
        pub fn unvote_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            Self::do_unregister_vote(origin, proposal_id)
        }

        // ---------------------------------
        // Profit sharing
        // ---------------------------------

        #[pallet::call_index(22)]
        #[pallet::weight((T::WeightInfo::add_profit_shares(), DispatchClass::Normal, Pays::No))]
        pub fn add_profit_shares(
            origin: OriginFor<T>,
            keys: Vec<T::AccountId>,
            shares: Vec<u16>,
        ) -> DispatchResult {
            Self::do_add_profit_shares(origin, keys, shares)
        }

        // ---------------------------------
        // Testnet
        // ---------------------------------

        #[pallet::call_index(23)]
        #[pallet::weight((Weight::from_parts(85_000_000, 0)
        .saturating_add(T::DbWeight::get().reads(16))
        .saturating_add(T::DbWeight::get().writes(28)), DispatchClass::Operational, Pays::No))]
        pub fn faucet(
            origin: OriginFor<T>,
            block_number: u64,
            nonce: u64,
            work: Vec<u8>,
        ) -> DispatchResult {
            if cfg!(feature = "testnet-faucet") {
                Self::do_faucet(origin, block_number, nonce, work)
            } else {
                Err(Error::<T>::FaucetDisabled.into())
            }
        }
    }

    // ---- Subspace helper functions.
    impl<T: Config> Pallet<T> {
        // --- Returns the transaction priority for setting weights.
        pub fn get_priority_set_weights(key: &T::AccountId, netuid: u16) -> u64 {
            if Uids::<T>::contains_key(netuid, key) {
                let uid: u16 = Self::get_uid_for_key(netuid, &key.clone());
                let current_block_number: u64 = Self::get_current_block_number();
                return current_block_number - Self::get_last_update_for_uid(netuid, uid);
            }
            0
        }
        // --- Returns the transaction priority for setting weights.
        pub fn get_priority_stake(key: &T::AccountId, netuid: u16) -> u64 {
            if Uids::<T>::contains_key(netuid, key) {
                return Stake::<T>::get(key);
            }
            0
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub enum CallType {
    SetWeights,
    AddStake,
    TransferStakeMultiple,
    TransferMultiple,
    TransferStake,
    AddStakeMultiple,
    RemoveStakeMultiple,
    RemoveStake,
    AddDelegate,
    Register,
    AddNetwork,
    Serve,
    #[default]
    Other,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct SubspaceSignedExtension<T: Config + Send + Sync + TypeInfo>(pub PhantomData<T>);

impl<T: Config + Send + Sync + TypeInfo> Default for SubspaceSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Config + Send + Sync + TypeInfo> SubspaceSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get_priority_vanilla(who: &T::AccountId) -> u64 {
        // Return high priority so that every extrinsic except set_weights function will
        // have a higher priority than the set_weights call
        // get the current block number
        let current_block_number: u64 = Pallet::<T>::get_current_block_number();
        let balance = Pallet::<T>::get_balance_u64(who);

        // this is the current block number minus the last update block number
        current_block_number + balance
    }

    pub fn get_priority_set_weights(who: &T::AccountId, netuid: u16) -> u64 {
        // Return the non vanilla priority for a set weights call.
        Pallet::<T>::get_priority_set_weights(who, netuid)
    }

    #[must_use]
    pub fn u64_to_balance(
        input: u64,
    ) -> Option<
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
    > {
        input.try_into().ok()
    }
}

impl<T: Config + Send + Sync + TypeInfo> sp_std::fmt::Debug for SubspaceSignedExtension<T> {
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "SubspaceSignedExtension")
    }
}

impl<T: Config + Send + Sync + TypeInfo> SignedExtension for SubspaceSignedExtension<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "SubspaceSignedExtension";

    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = (CallType, u64, Self::AccountId);

    fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        match call.is_sub_type() {
            Some(Call::set_weights { netuid, .. }) => {
                let priority: u64 = Self::get_priority_set_weights(who, *netuid);
                Ok(ValidTransaction {
                    priority,
                    longevity: 1,
                    ..Default::default()
                })
            }
            _ => Ok(ValidTransaction {
                priority: Self::get_priority_vanilla(who),
                ..Default::default()
            }),
        }
    }

    // NOTE: Add later when we put in a pre and post dispatch step.
    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let who = who.clone();
        match call.is_sub_type() {
            Some(Call::add_stake { .. }) => Ok((CallType::AddStake, 0, who)),
            Some(Call::add_stake_multiple { .. }) => Ok((CallType::AddStakeMultiple, 0, who)),
            Some(Call::remove_stake { .. }) => Ok((CallType::RemoveStake, 0, who)),
            Some(Call::remove_stake_multiple { .. }) => Ok((CallType::RemoveStakeMultiple, 0, who)),
            Some(Call::transfer_stake { .. }) => Ok((CallType::TransferStake, 0, who)),
            Some(Call::transfer_multiple { .. }) => Ok((CallType::TransferMultiple, 0, who)),
            Some(Call::set_weights { .. }) => Ok((CallType::SetWeights, 0, who)),
            Some(Call::register { .. }) => Ok((CallType::Register, 0, who)),
            Some(Call::update_module { .. }) => Ok((CallType::Serve, 0, who)),
            _ => Ok((CallType::Other, 0, who)),
        }
    }

    fn post_dispatch(
        maybe_pre: Option<Self::Pre>,
        _info: &DispatchInfoOf<Self::Call>,
        _post_info: &PostDispatchInfoOf<Self::Call>,
        _len: usize,
        _result: &dispatch::DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if let Some((call_type, _transaction_fee, _who)) = maybe_pre {
            match call_type {
                CallType::SetWeights => {
                    log::debug!("Not Implemented!");
                }
                CallType::AddStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }

                CallType::AddStakeMultiple => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::RemoveStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::RemoveStakeMultiple => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::TransferStake => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::TransferStakeMultiple => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::TransferMultiple => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::AddNetwork => {
                    log::debug!("Not Implemented! Need to add potential transaction fees here.");
                }
                CallType::Register => {
                    log::debug!("Not Implemented!");
                }
                _ => {
                    log::debug!("Not Implemented!");
                }
            }
        }
        Ok(())
    }
}
