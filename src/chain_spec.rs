use primitives::{Ed25519AuthorityId, ed25519};
use joystream_node_runtime::{
	AccountId, GenesisConfig, ConsensusConfig, TimestampConfig, BalancesConfig,
	SudoConfig, IndicesConfig, SessionConfig, StakingConfig, Permill, Perbill,
	CouncilConfig, CouncilElectionConfig, ProposalsConfig,
};
use substrate_service;
use hex_literal::{hex, hex_impl};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
	/// Staging testnet
	StagingTestnet
}

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| testnet_genesis(vec![
					ed25519::Pair::from_seed(b"Alice                           ").public().into(),
				], vec![
					ed25519::Pair::from_seed(b"Alice                           ").public().0.into(),
				],
					ed25519::Pair::from_seed(b"Alice                           ").public().0.into()
				),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| testnet_genesis(vec![
					ed25519::Pair::from_seed(b"Alice                           ").public().into(),
					ed25519::Pair::from_seed(b"Bob                             ").public().into(),
				], vec![
					ed25519::Pair::from_seed(b"Alice                           ").public().0.into(),
					ed25519::Pair::from_seed(b"Bob                             ").public().0.into(),
					ed25519::Pair::from_seed(b"Charlie                         ").public().0.into(),
					ed25519::Pair::from_seed(b"Dave                            ").public().0.into(),
					ed25519::Pair::from_seed(b"Eve                             ").public().0.into(),
					ed25519::Pair::from_seed(b"Ferdie                          ").public().0.into(),
				],
					ed25519::Pair::from_seed(b"Alice                           ").public().0.into()
				),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::StagingTestnet => staging_testnet_config(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"" | "local" => Some(Alternative::LocalTestnet),
			"staging" => Some(Alternative::StagingTestnet),
			_ => None,
		}
	}
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![
		String::from("/ip4/testnet-boot.joystream.org/tcp/30333/p2p/QmeuMS9ifbSbV3Sd9tEWyaVVDe85mPfcPWcTpp3LEcEQ53")
	];
	ChainSpec::from_genesis(
		"Joystream staging testnet",
		"joystream_staging_testnet",
		staging_testnet_config_genesis,
		boot_nodes,
		Some(STAGING_TELEMETRY_URL.into()),
		None,
		None,
		None,
	)
}

fn staging_testnet_config_genesis () -> GenesisConfig {
	let initial_authorities = vec![
		hex!["313ef1233684209e8b9740be3da31ac588874efae4b59771863afd44c2b620c4"].into(),
		//hex!["80c696c19b597e7cfba9135600a15735f789ae81f251826ecd6799d06164c15b"].into(),
	];
	let endowed_accounts = vec![
		hex!["2102ee83045058ba0f5e18bbc906437776c05771a2fc5915ff21c6ab76f41c31"].into(),
	];
	const MILLICENTS: u128 = 1_000_000_000;
	const CENTS: u128 = 1_000 * MILLICENTS;    // assume this is worth about a cent.
	const DOLLARS: u128 = 100 * CENTS;

	const SECS_PER_BLOCK: u64 = 6;
	const MINUTES: u64 = 60 / SECS_PER_BLOCK;
	const HOURS: u64 = MINUTES * 60;
	const DAYS: u64 = HOURS * 24;

	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/joystream_node_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			period: SECS_PER_BLOCK / 2, // due to the nature of aura the slots are 2*period
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().map(|&k| (k, 10_000_000 * DOLLARS)).collect(),
			transaction_base_fee: 1 * CENTS,
			transaction_byte_fee: 10 * MILLICENTS,
			existential_deposit: 1 * MILLICENTS,
			transfer_fee: 1 * MILLICENTS,
			creation_fee: 1 * MILLICENTS,
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().cloned().map(Into::into).collect(),
			session_length: 5 * MINUTES,
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			intentions: initial_authorities.iter().cloned().map(Into::into).collect(),
			offline_slash: Perbill::from_millionths(1000),
			session_reward: Perbill::from_billionths(2_065),
			current_offline_slash: 0,
			current_session_reward: 0,
			validator_count: 10,
			sessions_per_era: 12,
			bonding_duration: 60 * MINUTES,
			offline_slash_grace: 4,
			minimum_validator_count: 1,
			invulnerables: initial_authorities.iter().cloned().map(Into::into).collect(),
		}),
		council: Some(CouncilConfig {
			active_council: vec![],
			term_ends_at: 1,
		}),
		election: Some(CouncilElectionConfig {
			auto_start: false,
			announcing_period: 3 * DAYS,
			voting_period: 1 * DAYS,
			revealing_period: 1 * DAYS,
			council_size: 6,
			candidacy_limit: 25,
			min_council_stake: 1000,
			new_term_duration: 14 * DAYS,
			min_voting_stake: 10,
		}),
		proposals: Some(ProposalsConfig {
			approval_quorum: 60,
			minimum_stake: 100,
			cancellation_fee: 5,
			rejection_fee: 10,
			voting_period: 2 * DAYS,
			name_max_len: 32,
			description_max_len: 10_000,
			wasm_code_max_len: 2_000_000,
		}),

	}
}

fn testnet_genesis(initial_authorities: Vec<Ed25519AuthorityId>, endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/joystream_node_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			period: 3,                    // 3*2=6 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			existential_deposit: 500,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().map(|&k|(k, (1 << 60))).collect(),
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().cloned().map(Into::into).collect(),
			session_length: 10,
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			intentions: initial_authorities.iter().cloned().map(Into::into).collect(),
			minimum_validator_count: 1,
			validator_count: 2,
			sessions_per_era: 5,
			bonding_duration: 2 * 60 * 12,
			offline_slash: Perbill::zero(),
			session_reward: Perbill::zero(),
			current_offline_slash: 0,
			current_session_reward: 0,
			offline_slash_grace: 0,
			invulnerables: initial_authorities.iter().cloned().map(Into::into).collect(),
		}),
		council: Some(CouncilConfig {
			active_council: vec![],
			term_ends_at: 1,
		}),
		election: Some(CouncilElectionConfig {
			auto_start: false,
			announcing_period: 50,
			voting_period: 50,
			revealing_period: 50,
			council_size: 2,
			candidacy_limit: 25,
			min_council_stake: 100,
			new_term_duration: 1000,
			min_voting_stake: 10,
		}),
		proposals: Some(ProposalsConfig {
			approval_quorum: 60,
			minimum_stake: 100,
			cancellation_fee: 5,
			rejection_fee: 10,
			voting_period: 100,
			name_max_len: 100,
			description_max_len: 10_000,
			wasm_code_max_len: 2_000_000,
		}),
	}
}
