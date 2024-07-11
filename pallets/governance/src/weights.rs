
//! Autogenerated weights for `pallet_governance`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-07-11, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `DESKTOP-V6KUNF2`, CPU: `AMD Ryzen 9 5900X 12-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("specs/benchmarks.json")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/node-subspace
// benchmark
// pallet
// --chain
// specs/benchmarks.json
// --pallet
// pallet_governance
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --output
// pallets/governance/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_governance`.
pub trait WeightInfo {
	fn add_global_params_proposal() -> Weight;
	fn add_subnet_params_proposal() -> Weight;
	fn add_global_custom_proposal() -> Weight;
	fn add_subnet_custom_proposal() -> Weight;
	fn add_transfer_dao_treasury_proposal() -> Weight;
	fn vote_proposal() -> Weight;
	fn remove_vote_proposal() -> Weight;
	fn enable_vote_power_delegation() -> Weight;
	fn disable_vote_power_delegation() -> Weight;
	fn add_dao_application() -> Weight;
	fn refuse_dao_application() -> Weight;
	fn add_to_whitelist() -> Weight;
	fn remove_from_whitelist() -> Weight;
}

/// Weights for `pallet_governance` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `SubspaceModule::MaxNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MinNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedSubnets` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedModules` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedModules` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::FloorFounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorFounderShare` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorDelegationFee` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorDelegationFee` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerBlock` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BurnConfig` (r:1 w:0)
	/// Proof: `SubspaceModule::BurnConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeightsGlobal` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeightsGlobal` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Kappa` (r:1 w:0)
	/// Proof: `SubspaceModule::Kappa` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Rho` (r:1 w:0)
	/// Proof: `SubspaceModule::Rho` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_global_params_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1250`
		//  Estimated: `3569`
		// Minimum execution time: 56_547_000 picoseconds.
		Weight::from_parts(58_310_000, 3569)
			.saturating_add(T::DbWeight::get().reads(17_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `SubspaceModule::Founder` (r:1 w:0)
	/// Proof: `SubspaceModule::Founder` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FounderShare` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Tempo` (r:1 w:0)
	/// Proof: `SubspaceModule::Tempo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ImmunityPeriod` (r:1 w:0)
	/// Proof: `SubspaceModule::ImmunityPeriod` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedUids` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedUids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxWeightAge` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxWeightAge` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MinAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::SubnetNames` (r:6 w:0)
	/// Proof: `SubspaceModule::SubnetNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TrustRatio` (r:1 w:0)
	/// Proof: `SubspaceModule::TrustRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::IncentiveRatio` (r:1 w:0)
	/// Proof: `SubspaceModule::IncentiveRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (r:1 w:0)
	/// Proof: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BondsMovingAverage` (r:1 w:0)
	/// Proof: `SubspaceModule::BondsMovingAverage` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TargetRegistrationsInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::TargetRegistrationsInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TargetRegistrationsPerInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::TargetRegistrationsPerInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::AdjustmentAlpha` (r:1 w:0)
	/// Proof: `SubspaceModule::AdjustmentAlpha` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinImmunityStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinImmunityStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::SubnetGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::SubnetGovernanceConfig` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::MaxNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MinNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedSubnets` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedModules` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedModules` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::FloorFounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorFounderShare` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorDelegationFee` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorDelegationFee` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerBlock` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BurnConfig` (r:1 w:0)
	/// Proof: `SubspaceModule::BurnConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeightsGlobal` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeightsGlobal` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Kappa` (r:1 w:0)
	/// Proof: `SubspaceModule::Kappa` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Rho` (r:1 w:0)
	/// Proof: `SubspaceModule::Rho` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `SubnetEmissionModule::SubnetConsensusType` (r:1 w:0)
	/// Proof: `SubnetEmissionModule::SubnetConsensusType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_subnet_params_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2624`
		//  Estimated: `18464`
		// Minimum execution time: 146_016_000 picoseconds.
		Weight::from_parts(149_863_000, 18464)
			.saturating_add(T::DbWeight::get().reads(43_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_global_custom_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3569`
		// Minimum execution time: 28_734_000 picoseconds.
		Weight::from_parts(29_827_000, 3569)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_subnet_custom_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3569`
		// Minimum execution time: 33_463_000 picoseconds.
		Weight::from_parts(35_006_000, 3569)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::DaoTreasuryAddress` (r:1 w:0)
	/// Proof: `GovernanceModule::DaoTreasuryAddress` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_transfer_dao_treasury_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `324`
		//  Estimated: `6148`
		// Minimum execution time: 36_178_000 picoseconds.
		Weight::from_parts(37_000_000, 6148)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::StakeFrom` (r:2 w:0)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:0)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn vote_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1004`
		//  Estimated: `6944`
		// Minimum execution time: 35_467_000 picoseconds.
		Weight::from_parts(36_088_000, 6944)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn remove_vote_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `195`
		//  Estimated: `3464`
		// Minimum execution time: 14_277_000 picoseconds.
		Weight::from_parts(14_928_000, 3464)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:1)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn enable_vote_power_delegation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `43`
		//  Estimated: `1484`
		// Minimum execution time: 4_809_000 picoseconds.
		Weight::from_parts(5_009_000, 1484)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:1)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn disable_vote_power_delegation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `43`
		//  Estimated: `1484`
		// Minimum execution time: 5_040_000 picoseconds.
		Weight::from_parts(5_370_000, 1484)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:1 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	fn add_dao_application() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `120`
		//  Estimated: `3812`
		// Minimum execution time: 27_792_000 picoseconds.
		Weight::from_parts(28_374_000, 3812)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:1 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	fn refuse_dao_application() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `214`
		//  Estimated: `3812`
		// Minimum execution time: 10_369_000 picoseconds.
		Weight::from_parts(10_871_000, 3812)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:2 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::LegitWhitelist` (r:1 w:1)
	/// Proof: `GovernanceModule::LegitWhitelist` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	fn add_to_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `291`
		//  Estimated: `6634`
		// Minimum execution time: 35_367_000 picoseconds.
		Weight::from_parts(36_459_000, 6634)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::LegitWhitelist` (r:1 w:1)
	/// Proof: `GovernanceModule::LegitWhitelist` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	fn remove_from_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `161`
		//  Estimated: `3498`
		// Minimum execution time: 11_733_000 picoseconds.
		Weight::from_parts(12_093_000, 3498)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `SubspaceModule::MaxNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MinNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedSubnets` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedModules` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedModules` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::FloorFounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorFounderShare` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorDelegationFee` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorDelegationFee` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerBlock` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BurnConfig` (r:1 w:0)
	/// Proof: `SubspaceModule::BurnConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeightsGlobal` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeightsGlobal` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Kappa` (r:1 w:0)
	/// Proof: `SubspaceModule::Kappa` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Rho` (r:1 w:0)
	/// Proof: `SubspaceModule::Rho` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_global_params_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1250`
		//  Estimated: `3569`
		// Minimum execution time: 56_547_000 picoseconds.
		Weight::from_parts(58_310_000, 3569)
			.saturating_add(RocksDbWeight::get().reads(17_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `SubspaceModule::Founder` (r:1 w:0)
	/// Proof: `SubspaceModule::Founder` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FounderShare` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Tempo` (r:1 w:0)
	/// Proof: `SubspaceModule::Tempo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ImmunityPeriod` (r:1 w:0)
	/// Proof: `SubspaceModule::ImmunityPeriod` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedUids` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedUids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxWeightAge` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxWeightAge` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MinAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::SubnetNames` (r:6 w:0)
	/// Proof: `SubspaceModule::SubnetNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TrustRatio` (r:1 w:0)
	/// Proof: `SubspaceModule::TrustRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::IncentiveRatio` (r:1 w:0)
	/// Proof: `SubspaceModule::IncentiveRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (r:1 w:0)
	/// Proof: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BondsMovingAverage` (r:1 w:0)
	/// Proof: `SubspaceModule::BondsMovingAverage` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TargetRegistrationsInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::TargetRegistrationsInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TargetRegistrationsPerInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::TargetRegistrationsPerInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerInterval` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::AdjustmentAlpha` (r:1 w:0)
	/// Proof: `SubspaceModule::AdjustmentAlpha` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinImmunityStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinImmunityStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::SubnetGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::SubnetGovernanceConfig` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::MaxNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MinNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedSubnets` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedModules` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedModules` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::FloorFounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorFounderShare` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorDelegationFee` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorDelegationFee` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerBlock` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BurnConfig` (r:1 w:0)
	/// Proof: `SubspaceModule::BurnConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeightsGlobal` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeightsGlobal` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Kappa` (r:1 w:0)
	/// Proof: `SubspaceModule::Kappa` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Rho` (r:1 w:0)
	/// Proof: `SubspaceModule::Rho` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `SubnetEmissionModule::SubnetConsensusType` (r:1 w:0)
	/// Proof: `SubnetEmissionModule::SubnetConsensusType` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_subnet_params_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2624`
		//  Estimated: `18464`
		// Minimum execution time: 146_016_000 picoseconds.
		Weight::from_parts(149_863_000, 18464)
			.saturating_add(RocksDbWeight::get().reads(43_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_global_custom_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3569`
		// Minimum execution time: 28_734_000 picoseconds.
		Weight::from_parts(29_827_000, 3569)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_subnet_custom_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `126`
		//  Estimated: `3569`
		// Minimum execution time: 33_463_000 picoseconds.
		Weight::from_parts(35_006_000, 3569)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::DaoTreasuryAddress` (r:1 w:0)
	/// Proof: `GovernanceModule::DaoTreasuryAddress` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::GlobalGovernanceConfig` (r:1 w:0)
	/// Proof: `GovernanceModule::GlobalGovernanceConfig` (`max_values`: Some(1), `max_size`: Some(30), added: 525, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn add_transfer_dao_treasury_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `324`
		//  Estimated: `6148`
		// Minimum execution time: 36_178_000 picoseconds.
		Weight::from_parts(37_000_000, 6148)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::StakeFrom` (r:2 w:0)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:0)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn vote_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1004`
		//  Estimated: `6944`
		// Minimum execution time: 35_467_000 picoseconds.
		Weight::from_parts(36_088_000, 6944)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::Proposals` (r:1 w:1)
	/// Proof: `GovernanceModule::Proposals` (`max_values`: None, `max_size`: Some(4294967295), added: 2474, mode: `MaxEncodedLen`)
	fn remove_vote_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `195`
		//  Estimated: `3464`
		// Minimum execution time: 14_277_000 picoseconds.
		Weight::from_parts(14_928_000, 3464)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:1)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn enable_vote_power_delegation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `43`
		//  Estimated: `1484`
		// Minimum execution time: 4_809_000 picoseconds.
		Weight::from_parts(5_009_000, 1484)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::NotDelegatingVotingPower` (r:1 w:1)
	/// Proof: `GovernanceModule::NotDelegatingVotingPower` (`max_values`: Some(1), `max_size`: Some(4294967295), added: 494, mode: `MaxEncodedLen`)
	fn disable_vote_power_delegation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `43`
		//  Estimated: `1484`
		// Minimum execution time: 5_040_000 picoseconds.
		Weight::from_parts(5_370_000, 1484)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `GovernanceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:1 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	fn add_dao_application() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `120`
		//  Estimated: `3812`
		// Minimum execution time: 27_792_000 picoseconds.
		Weight::from_parts(28_374_000, 3812)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:1 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	fn refuse_dao_application() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `214`
		//  Estimated: `3812`
		// Minimum execution time: 10_369_000 picoseconds.
		Weight::from_parts(10_871_000, 3812)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::CuratorApplications` (r:2 w:1)
	/// Proof: `GovernanceModule::CuratorApplications` (`max_values`: None, `max_size`: Some(347), added: 2822, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::LegitWhitelist` (r:1 w:1)
	/// Proof: `GovernanceModule::LegitWhitelist` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	fn add_to_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `291`
		//  Estimated: `6634`
		// Minimum execution time: 35_367_000 picoseconds.
		Weight::from_parts(36_459_000, 6634)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `GovernanceModule::Curator` (r:1 w:0)
	/// Proof: `GovernanceModule::Curator` (`max_values`: Some(1), `max_size`: Some(32), added: 527, mode: `MaxEncodedLen`)
	/// Storage: `GovernanceModule::LegitWhitelist` (r:1 w:1)
	/// Proof: `GovernanceModule::LegitWhitelist` (`max_values`: None, `max_size`: Some(33), added: 2508, mode: `MaxEncodedLen`)
	fn remove_from_whitelist() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `161`
		//  Estimated: `3498`
		// Minimum execution time: 11_733_000 picoseconds.
		Weight::from_parts(12_093_000, 3498)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}