
//! Autogenerated weights for `pallet_subspace`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-05-18, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Ubuntu-2204-jammy-amd64-base`, CPU: `AMD Ryzen 9 7950X3D 16-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("specs/benchmark.json")`, DB CACHE: 1024

// Executed Command:
// ./target/release/node-subspace
// benchmark
// pallet
// --chain
// specs/benchmark.json
// --pallet
// pallet_subspace
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --output
// pallets/subspace/src/autogen_benchmark.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_subspace`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_subspace::WeightInfo for WeightInfo<T> {
	/// Storage: `SubspaceModule::RegistrationsPerBlock` (r:1 w:1)
	/// Proof: `SubspaceModule::RegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxRegistrationsPerBlock` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxRegistrationsPerBlock` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::SubnetNames` (r:1 w:1)
	/// Proof: `SubspaceModule::SubnetNames` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorFounderShare` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorFounderShare` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinNameLength` (r:1 w:0)
	/// Proof: `SubspaceModule::MinNameLength` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedSubnets` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedModules` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedModules` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::UnitEmission` (r:1 w:0)
	/// Proof: `SubspaceModule::UnitEmission` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Curator` (r:1 w:0)
	/// Proof: `SubspaceModule::Curator` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FloorDelegationFee` (r:1 w:0)
	/// Proof: `SubspaceModule::FloorDelegationFee` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeightsGlobal` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeightsGlobal` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::SubnetStakeThreshold` (r:1 w:0)
	/// Proof: `SubspaceModule::SubnetStakeThreshold` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ProposalCost` (r:1 w:0)
	/// Proof: `SubspaceModule::ProposalCost` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ProposalExpiration` (r:1 w:0)
	/// Proof: `SubspaceModule::ProposalExpiration` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ProposalParticipationThreshold` (r:1 w:0)
	/// Proof: `SubspaceModule::ProposalParticipationThreshold` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::GeneralSubnetApplicationCost` (r:1 w:0)
	/// Proof: `SubspaceModule::GeneralSubnetApplicationCost` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::BurnConfig` (r:1 w:0)
	/// Proof: `SubspaceModule::BurnConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalSubnets` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalSubnets` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::SubnetGaps` (r:1 w:1)
	/// Proof: `SubspaceModule::SubnetGaps` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::N` (r:2 w:1)
	/// Proof: `SubspaceModule::N` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Uids` (r:1 w:1)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Keys` (r:1 w:1)
	/// Proof: `SubspaceModule::Keys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Name` (r:1 w:1)
	/// Proof: `SubspaceModule::Name` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Active` (r:1 w:1)
	/// Proof: `SubspaceModule::Active` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Consensus` (r:1 w:1)
	/// Proof: `SubspaceModule::Consensus` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Emission` (r:1 w:1)
	/// Proof: `SubspaceModule::Emission` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Incentive` (r:1 w:1)
	/// Proof: `SubspaceModule::Incentive` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Dividends` (r:1 w:1)
	/// Proof: `SubspaceModule::Dividends` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::LastUpdate` (r:1 w:1)
	/// Proof: `SubspaceModule::LastUpdate` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::PruningScores` (r:1 w:1)
	/// Proof: `SubspaceModule::PruningScores` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Rank` (r:1 w:1)
	/// Proof: `SubspaceModule::Rank` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Trust` (r:1 w:1)
	/// Proof: `SubspaceModule::Trust` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ValidatorPermits` (r:1 w:1)
	/// Proof: `SubspaceModule::ValidatorPermits` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ValidatorTrust` (r:1 w:1)
	/// Proof: `SubspaceModule::ValidatorTrust` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeFrom` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeTo` (r:2 w:2)
	/// Proof: `SubspaceModule::StakeTo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Stake` (r:1 w:1)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalStake` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::RegistrationsThisInterval` (r:1 w:1)
	/// Proof: `SubspaceModule::RegistrationsThisInterval` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Burn` (r:0 w:1)
	/// Proof: `SubspaceModule::Burn` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::RegistrationBlock` (r:0 w:1)
	/// Proof: `SubspaceModule::RegistrationBlock` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::DelegationFee` (r:0 w:1)
	/// Proof: `SubspaceModule::DelegationFee` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxWeightAge` (r:0 w:1)
	/// Proof: `SubspaceModule::MaxWeightAge` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TrustRatio` (r:0 w:1)
	/// Proof: `SubspaceModule::TrustRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Address` (r:0 w:1)
	/// Proof: `SubspaceModule::Address` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::VoteModeSubnet` (r:0 w:1)
	/// Proof: `SubspaceModule::VoteModeSubnet` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinAllowedWeights` (r:0 w:1)
	/// Proof: `SubspaceModule::MinAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::IncentiveRatio` (r:0 w:1)
	/// Proof: `SubspaceModule::IncentiveRatio` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeights` (r:0 w:1)
	/// Proof: `SubspaceModule::MaxAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Tempo` (r:0 w:1)
	/// Proof: `SubspaceModule::Tempo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinStake` (r:0 w:1)
	/// Proof: `SubspaceModule::MinStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::FounderShare` (r:0 w:1)
	/// Proof: `SubspaceModule::FounderShare` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Founder` (r:0 w:1)
	/// Proof: `SubspaceModule::Founder` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Metadata` (r:0 w:1)
	/// Proof: `SubspaceModule::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::ImmunityPeriod` (r:0 w:1)
	/// Proof: `SubspaceModule::ImmunityPeriod` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::SubnetEmission` (r:0 w:1)
	/// Proof: `SubspaceModule::SubnetEmission` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedUids` (r:0 w:1)
	/// Proof: `SubspaceModule::MaxAllowedUids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (r:0 w:1)
	/// Proof: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `119`
		//  Estimated: `6059`
		// Minimum execution time: 143_237_000 picoseconds.
		Weight::from_parts(145_682_000, 0)
			.saturating_add(Weight::from_parts(0, 6059))
			.saturating_add(T::DbWeight::get().reads(44))
			.saturating_add(T::DbWeight::get().writes(45))
	}
	/// Storage: `SubspaceModule::N` (r:1 w:0)
	/// Proof: `SubspaceModule::N` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Uids` (r:1 w:0)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (r:1 w:0)
	/// Proof: `SubspaceModule::MaximumSetWeightCallsPerEpoch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Keys` (r:1 w:0)
	/// Proof: `SubspaceModule::Keys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MinAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MaxAllowedWeights` (r:1 w:0)
	/// Proof: `SubspaceModule::MaxAllowedWeights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Stake` (r:1 w:0)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::MinWeightStake` (r:1 w:0)
	/// Proof: `SubspaceModule::MinWeightStake` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::LastUpdate` (r:1 w:1)
	/// Proof: `SubspaceModule::LastUpdate` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Weights` (r:0 w:1)
	/// Proof: `SubspaceModule::Weights` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_weights() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1193`
		//  Estimated: `4658`
		// Minimum execution time: 38_753_000 picoseconds.
		Weight::from_parts(40_285_000, 0)
			.saturating_add(Weight::from_parts(0, 4658))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `SubspaceModule::Uids` (r:1 w:0)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::StakeTo` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeTo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Stake` (r:1 w:1)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeFrom` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalStake` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1097`
		//  Estimated: `4562`
		// Minimum execution time: 45_284_000 picoseconds.
		Weight::from_parts(45_935_000, 0)
			.saturating_add(Weight::from_parts(0, 4562))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Uids` (r:2 w:0)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeTo` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeTo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Stake` (r:2 w:2)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeFrom` (r:2 w:2)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalStake` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_stake_multiple() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1379`
		//  Estimated: `7319`
		// Minimum execution time: 82_514_000 picoseconds.
		Weight::from_parts(83_796_000, 0)
			.saturating_add(Weight::from_parts(0, 7319))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
	/// Storage: `SubspaceModule::Uids` (r:1 w:0)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeTo` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeTo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Stake` (r:1 w:1)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeFrom` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalStake` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_stake() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1176`
		//  Estimated: `4641`
		// Minimum execution time: 46_066_000 picoseconds.
		Weight::from_parts(47_018_000, 0)
			.saturating_add(Weight::from_parts(0, 4641))
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	/// Storage: `SubspaceModule::StakeTo` (r:1 w:1)
	/// Proof: `SubspaceModule::StakeTo` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::Uids` (r:2 w:0)
	/// Proof: `SubspaceModule::Uids` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(104), added: 2579, mode: `MaxEncodedLen`)
	/// Storage: `SubspaceModule::Stake` (r:2 w:2)
	/// Proof: `SubspaceModule::Stake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::StakeFrom` (r:2 w:2)
	/// Proof: `SubspaceModule::StakeFrom` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `SubspaceModule::TotalStake` (r:1 w:1)
	/// Proof: `SubspaceModule::TotalStake` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn remove_stake_multiple() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1491`
		//  Estimated: `7431`
		// Minimum execution time: 83_756_000 picoseconds.
		Weight::from_parts(85_129_000, 0)
			.saturating_add(Weight::from_parts(0, 7431))
			.saturating_add(T::DbWeight::get().reads(9))
			.saturating_add(T::DbWeight::get().writes(7))
	}
}