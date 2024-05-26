#![allow(non_camel_case_types)]

use frame_support::{
    parameter_types,
    traits::{Currency, Everything, Get},
    PalletId,
};
use frame_system as system;
use sp_core::{H256, U256};
use std::cell::RefCell;

use pallet_subspace::DefaultKey;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        SubnetEmission: pallet_subnet_emission,
        Subspace: pallet_subspace,
    }
);

#[allow(dead_code)]
pub type BalanceCall = pallet_balances::Call<Test>;

#[allow(dead_code)]
pub type TestRuntimeCall = frame_system::Call<Test>;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

#[allow(dead_code)]
pub type AccountId = U256;

// Balance of an account.
pub type Balance = u64;

// An index to a block.
#[allow(dead_code)]
pub type BlockNumber = u64;

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

pub const PALLET_ID: PalletId = PalletId(*b"py/subsp");

pub struct SubspacePalletId;

impl Get<PalletId> for SubspacePalletId {
    fn get() -> PalletId {
        PALLET_ID
    }
}

impl pallet_subspace::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = ();
    type PalletId = SubspacePalletId;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AccountStore = System;
    type Balance = u64;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = ();
    type RuntimeHoldReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = frame_support::traits::ConstU32<16>;
    type RuntimeFreezeReason = ();
}

impl system::Config for Test {
    type BaseCallFilter = Everything;
    type Block = Block;
    type BlockWeights = ();
    type BlockLength = ();
    type AccountId = U256;
    type RuntimeCall = RuntimeCall;
    type Nonce = u32;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;

    type RuntimeTask = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

pub struct MockEmissionConfig {
    pub decimals: u8,
    pub halving_interval: u64,
    pub max_supply: u64,
}

impl Default for MockEmissionConfig {
    fn default() -> Self {
        Self {
            decimals: 12,
            halving_interval: 1_000_000,
            max_supply: 1_000_000_000,
        }
    }
}

impl pallet_subnet_emission::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Decimals = Decimals;
    type HalvingInterval = HalvingInterval;
    type MaxSupply = MaxSupply;
}

pub struct Decimals;
impl Get<u8> for Decimals {
    fn get() -> u8 {
        MOCK_EMISSION_CONFIG.with(|config| config.borrow().decimals)
    }
}

pub struct HalvingInterval;
impl Get<u64> for HalvingInterval {
    fn get() -> u64 {
        MOCK_EMISSION_CONFIG.with(|config| config.borrow().halving_interval)
    }
}

pub struct MaxSupply;
impl Get<u64> for MaxSupply {
    fn get() -> u64 {
        MOCK_EMISSION_CONFIG.with(|config| config.borrow().max_supply)
    }
}

thread_local! {
    static MOCK_EMISSION_CONFIG: RefCell<MockEmissionConfig> = RefCell::new(MockEmissionConfig::default());
}

pub fn set_emission_config(decimals: u8, halving_interval: u64, max_supply: u64) {
    MOCK_EMISSION_CONFIG.with(|config| {
        *config.borrow_mut() = MockEmissionConfig {
            decimals,
            halving_interval,
            max_supply,
        };
    });
}

pub fn set_total_issuance(total_issuance: u64) {
    let key = DefaultKey::<Test>::get();
    // Reset the issuance (completelly nuke the key's balance)
    <Test as pallet_subspace::Config>::Currency::make_free_balance_be(&key, 0u32.into());
    // Add the total_issuance to the key's balance
    Subspace::add_balance_to_account(&key, total_issuance);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
