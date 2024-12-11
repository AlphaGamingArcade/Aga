use aga_runtime::{AccountId, RuntimeGenesisConfig, Signature, WASM_BINARY, Balance, opaque::SessionKeys };
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ed25519, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::str::FromStr;
use serde_json::json;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

// pub fn aga_config() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../resources/aga-chain-spec-raw.json")[..])
// }

/// Generate a crypto pair from seed.
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

// /// Generate an Aura authority key.
// pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
// 	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
// }

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s)
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_protocol_id("aga")
	.with_genesis_config_patch(testnet_genesis(
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
	))
	.with_properties(default_properties())
	.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_protocol_id("aga")
	.with_genesis_config_patch(testnet_genesis(
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
	))
	.with_properties(default_properties())
	.build())
}

pub fn aga_testnet_config() -> Result<ChainSpec, String> {
	const ROOT_PUBLIC_SR25519: &str = "5HTjThDzsZuY7neTGTZrgJGKADHjyW9R4W3emZYJwZPa91gS";

	const NODE1_PUBLIC_SR25519: &str = "5GNJEdcdXmMyN7c95ZbeWtJsiZoSv7kVVVDvxfA5TnL7qA9k";
	const NODE1_PUBLIC_ED25519: &str = "5F7wpC4yLPC5pymEDKwSvmkfuqbKLFdwB9qbg8E4UEvqkoBE";

	const NODE2_PUBLIC_SR25519: &str = "5H9Pr92kVX4wAmuuAEbVHHoDnccF4jZthaTcrfrYR2Q8Yh1u";
	const NODE2_PUBLIC_ED25519: &str = "5DJxo9qjo7o5Ses674Pe7aVojDqtWLYH4D5d8K5JmEoShDri";

	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Aga Testnet")
	.with_id("aga_testnet")
	.with_chain_type(ChainType::Live)
	.with_protocol_id("aga")
	.with_genesis_config_patch(testnet_genesis(
		// Initial PoA authorities
		vec![
			(
				AccountId::from_str(NODE1_PUBLIC_SR25519).unwrap(),
				AuraId::from(sr25519::Public::from_str(NODE1_PUBLIC_SR25519).unwrap()),
				GrandpaId::from(ed25519::Public::from_str(NODE1_PUBLIC_ED25519).unwrap()),
			),
			(
				AccountId::from_str(NODE2_PUBLIC_SR25519).unwrap(),
				AuraId::from(sr25519::Public::from_str(NODE2_PUBLIC_SR25519).unwrap()),
				GrandpaId::from(ed25519::Public::from_str(NODE2_PUBLIC_ED25519).unwrap()),
			)
		],
		// Sudo account
		AccountId::from_str(ROOT_PUBLIC_SR25519).unwrap(),
		// Pre-funded accounts
		vec![
			AccountId::from_str(ROOT_PUBLIC_SR25519).unwrap()
		],
		true,
	))
	.with_properties(default_properties())
	.build())
}

const INITIAL_BALANCE: Balance = 100_000_000_000_000_000_000_000;

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			// Configure endowed accounts with initial balance.
			"balances": endowed_accounts.iter().cloned().map(|k| (k, INITIAL_BALANCE)).collect::<Vec<_>>(),
		},
		"validatorSet": {
			"initialValidators": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		"session": {
			"keys": initial_authorities
				.iter()
				.map(|x| (
					x.0.clone(),
					x.0.clone(),
					session_keys(x.1.clone(), x.2.clone())
				))
				.collect::<Vec<_>>(),
		},
		"aura": {
			"authorities": [],
		},
		"grandpa": {
			"authorities": [],
		},
		"sudo": {
			"key": Some(root_key),
		}
	})
}

fn default_properties() -> sc_service::Properties {
	let mut props : sc_service::Properties = sc_service::Properties::new();
	props.insert("tokenSymbol".to_string(), json!("AGAT"));
	props.insert("tokenDecimals".to_string(), json!(18));
	return props;
}