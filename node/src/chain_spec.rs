use node_subspace_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, SubspaceModuleConfig
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::crypto::Ss58Codec;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn authority_keys_from_ss58(s_aura :&str, s_grandpa : &str) -> (AuraId, GrandpaId) {
	(
		get_aura_from_ss58_addr(s_aura),
		get_grandpa_from_ss58_addr(s_grandpa),
	)
}

pub fn get_aura_from_ss58_addr(s: &str) -> AuraId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn get_grandpa_from_ss58_addr(s: &str) -> GrandpaId {
	Ss58Codec::from_ss58check(s).unwrap()
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

// Includes for nakamoto genesis
use std::{fs::File, path::PathBuf};
use serde::{Deserialize};
use serde_json as json;

// Configure storage from nakamoto data
pub fn netwon_config() -> Result<ChainSpec, String> {
	let path: PathBuf = std::path::PathBuf::from("./snapshot.json");
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	// We mmap the file into memory first, as this is *a lot* faster than using
	// `serde_json::from_reader`. See https://github.com/serde-rs/json/issues/160
	let file = File::open(&path)
		.map_err(|e| format!("Error opening genesis file `{}`: {}", path.display(), e))?;

	// SAFETY: `mmap` is fundamentally unsafe since technically the file can change
	//         underneath us while it is mapped; in practice it's unlikely to be a problem
	let bytes = unsafe {
		memmap2::Mmap::map(&file)
			.map_err(|e| format!("Error mmaping genesis file `{}`: {}", path.display(), e))?
	};


	let mut processed_stakes: Vec<(sp_runtime::AccountId32, Vec<(sp_runtime::AccountId32, (u64, u16))>)> = Vec::new();
	let mut balances_issuance: u64 = 0;
	let mut processed_balances: Vec<(sp_runtime::AccountId32, u64)> = Vec::new();

	// Give front-ends necessary data to present to users
	let mut properties = sc_service::Properties::new();
	properties.insert("tokenSymbol".into(), "MAO".into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("ss58Format".into(), 13116.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Commune",
		// ID
		"commune",
		ChainType::Development,
		move || {
			finney_genesis(
				wasm_binary,
				// Initial PoA authorities (Validators)
				// aura | grandpa
				vec![
					// Keys for debug
					authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"),
					], 
				// Sudo account
				Ss58Codec::from_ss58check("5G3rrAtAsZiZsRKhi9y7yckwhB6HL8AwD8K3J28YNX4n2tMZ").unwrap(), 
				// Pre-funded accounts
				vec![
				],
				true,
				processed_stakes.clone(),
				processed_balances.clone(),
				balances_issuance
			)
		},
		// Bootnodes
		vec![
		],
		// Telemetry
		None,
		// Protocol ID
		Some("commune"),
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		subspace_module: Default::default(),
	}
}

// Configure initial storage state for FRAME modules.
fn finney_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	_endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
	stakes: Vec<(AccountId, Vec<(AccountId, (u64, u16))>)>,
	balances: Vec<(AccountId, u64)>,
	balances_issuance: u64

) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			//balances: balances.iter().cloned().map(|k| k).collect(),
			balances: balances.iter().cloned().map(|k| k).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		subspace_module: SubspaceModuleConfig {
			stakes: stakes,
			balances_issuance: balances_issuance
		},
	}
}
