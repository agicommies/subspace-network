use node_subspace_runtime::{
    AccountId, AuraConfig, BalancesConfig, GrandpaConfig, Precompiles, RuntimeGenesisConfig,
    SubspaceModuleConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    <TPublic::Pair as Pair>::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

// Includes for nakamoto genesis
use serde::Deserialize;
use serde_json as json;
use std::{fs::File, path::PathBuf};

/// (name, tempo, immunity_period, min_allowed_weights, max_allowed_weights,
/// max_allowed_uids, founder)
pub type JSONSubnet = (String, u16, u16, u16, u16, u16, u16, u64, String);

/// (key, name, address, stake, weights)
pub type JSONModule = (String, String, String, Vec<(u16, u16)>);

/// (module_key, amount)
pub type JSONStakeTo = (String, Vec<(String, u64)>);

/// (name, tempo, immunity_period, min_allowed_weights, max_allowed_weights,
/// max_allowed_uids, founder)
pub type Subnet = (
    Vec<u8>,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u64,
    sp_runtime::AccountId32,
);

/// (key, name, address, stake, weights)
pub type Module = (sp_runtime::AccountId32, Vec<u8>, Vec<u8>, Vec<(u16, u16)>);

/// (module_key, amount)
pub type StakeTo = (sp_runtime::AccountId32, Vec<(sp_runtime::AccountId32, u64)>);

// Configure storage from nakamoto data
#[derive(Deserialize, Debug)]
struct SubspaceJSONState {
    balances: std::collections::HashMap<String, u64>,
    // subnet -> Subnet
    subnets: Vec<JSONSubnet>,

    // subnet -> module -> Module
    modules: Vec<Vec<JSONModule>>,

    // subnet -> key -> StakeTo
    stake_to: Vec<Vec<JSONStakeTo>>,

    // block at sync
    block: u32,

    // version
    #[allow(unused)]
    version: u32,
}

pub fn generate_config(network: String) -> Result<ChainSpec, String> {
    let path: PathBuf = std::path::PathBuf::from(format!("./snapshots/{}.json", network));
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

    let state: SubspaceJSONState =
        json::from_slice(&bytes).map_err(|e| format!("Error parsing genesis file: {}", e))?;

    let subnets: Vec<Subnet> = state.subnets.into_iter().map(deserialize_subnet).collect();
    let mut modules: Vec<Vec<Module>> = Vec::new();
    let mut stake_to: Vec<Vec<StakeTo>> = Vec::new();

    for netuid in 0..subnets.len() {
        // Add  modules
        modules.push(vec![]);
        for (id, name, address, weights) in state.modules[netuid].iter() {
            modules[netuid].push((
                new_account_id(id),
                name.as_bytes().to_vec(),
                address.as_bytes().to_vec(),
                weights.iter().map(|(a, b)| (*a, *b)).collect(),
            ));
        }

        // Add stake to
        stake_to.push(Vec::new());
        for (key, key_stake_to) in state.stake_to[netuid].iter() {
            stake_to[netuid].push((
                new_account_id(key),
                key_stake_to.iter().map(|(a, b)| (new_account_id(a), *b)).collect(),
            ));
        }
    }

    let processed_balances: Vec<(sp_runtime::AccountId32, u64)> = state
        .balances
        .iter()
        .map(|(id, amount)| (new_account_id(id), *amount))
        .collect();

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "C".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 13116.into());

    Ok(ChainSpec::from_genesis(
        "commune", // Name
        "commune", // ID
        ChainType::Development,
        move || {
            // Sudo account
            let root =
                Ss58Codec::from_ss58check("5FXymAnjbb7p57pNyfdLb6YCdzm73ZhVq6oFF1AdCEPEg8Uw")
                    .unwrap();

            // Initial PoA authorities (Validators)
            // aura | grandpa
            let initial_authorities = vec![
                // Keys for debug
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
            ];
            network_genesis(
                wasm_binary,
                initial_authorities,
                root,
                processed_balances.clone(),
                SubspaceModuleConfig {
                    // Add names to storage.
                    modules: modules.clone(),
                    subnets: subnets.clone(),
                    block: state.block,
                    stake_to: stake_to.clone(),
                },
            )
        },
        vec![],           // Bootnodes
        None,             // Telemetry
        Some("commune"),  // Protocol ID
        None,             //
        Some(properties), // Properties
        None,             // Extensions
    ))
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    generate_config("main".to_string())
}

pub fn testnet_config() -> Result<ChainSpec, String> {
    generate_config("test".to_string())
}

// Configure initial storage state for FRAME modules.
fn network_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    balances: Vec<(AccountId, u64)>,
    module: SubspaceModuleConfig,
) -> RuntimeGenesisConfig {
    use node_subspace_runtime::EVMConfig;

    let (aura, grandpa): (Vec<_>, Vec<_>) = initial_authorities
        .into_iter()
        .map(|(aura, grandpa)| (aura, (grandpa, 1)))
        .unzip();

    RuntimeGenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            ..Default::default()
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            //balances: balances.iter().cloned().map(|k| k).collect(),
            balances: balances.to_vec(),
        },
        aura: AuraConfig { authorities: aura },
        grandpa: GrandpaConfig {
            authorities: grandpa,
            ..Default::default()
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        transaction_payment: Default::default(),
        subspace_module: module,
        // EVM Compatibility
        evm_chain_id: Default::default(),
        evm: EVMConfig {
            accounts: Precompiles::used_addresses()
                .map(|addr| {
                    let account = fp_evm::GenesisAccount {
                        balance: Default::default(),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    };
                    (addr, account)
                })
                .collect(),
            _marker: Default::default(),
        },
        ethereum: Default::default(),
        base_fee: Default::default(),
    }
}

fn new_account_id(id: &str) -> sp_runtime::AccountId32 {
    sp_runtime::AccountId32::from(<sr25519::Public as Ss58Codec>::from_ss58check(id).unwrap())
}

fn deserialize_subnet(
    (
        name,
        tempo,
        immunity_period,
        min_allowed_weights,
        max_allowed_weights,
        max_allowed_uids,
        burn_rate,
        min_stake,
        founder,
    ): JSONSubnet,
) -> Subnet {
    (
        name.as_bytes().to_vec(),
        tempo,
        immunity_period,
        min_allowed_weights,
        max_allowed_weights,
        max_allowed_uids,
        burn_rate,
        min_stake,
        new_account_id(&founder),
    )
}
