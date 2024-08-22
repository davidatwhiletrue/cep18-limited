use casper_engine_test_support::utils::create_run_genesis_request;
use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, UpgradeRequestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_fixtures::{builder_from_global_state_fixture, generate_fixture, shrink_db};
use casper_types::addressable_entity::EntityKindTag;
use casper_types::bytesrepr::FromBytes;
use casper_types::{runtime_args, EraId, RuntimeArgs, U256};
use casper_types::{
    AddressableEntityHash, CLTyped, EntityAddr, GenesisAccount, Motes, PackageHash, U512,
};

use casper_types::{account::AccountHash, Key, PublicKey, SecretKey};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use utility::constants::{
    AMOUNT, ARG_ADDRESS, ARG_AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_OWNER, ARG_SYMBOL,
    ARG_TOKEN_CONTRACT, ARG_TOTAL_SUPPLY, CEP18_CONTRACT_WASM, CEP18_TEST_CONTRACT_KEY,
    CEP18_TEST_CONTRACT_WASM, CEP18_TOKEN_CONTRACT_KEY, CHECK_BALANCE_OF_ENTRYPOINT, OWNER,
    RESULT_KEY, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
};

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use casper_types::{Digest, ProtocolVersion};

const STATE_JSON_FILE: &str = "state.json";
const GENESIS_PROTOCOL_VERSION_FIELD: &str = "protocol_version";

fn path_to_lmdb_fixtures() -> PathBuf {
    Path::new("./fixtures/").to_path_buf()
}

/// Contains serialized genesis config.
#[derive(Serialize, Deserialize)]
pub struct LmdbFixtureState {
    /// Serializes as unstructured JSON value because [`GenesisRequest`] might change over time
    /// and likely old fixture might not deserialize cleanly in the future.
    pub genesis_request: serde_json::Value,
    pub post_state_hash: Digest,
}

impl LmdbFixtureState {
    pub fn genesis_protocol_version(&self) -> ProtocolVersion {
        serde_json::from_value(
            self.genesis_request
                .get(GENESIS_PROTOCOL_VERSION_FIELD)
                .cloned()
                .unwrap(),
        )
        .expect("should have protocol version field")
    }
}

#[allow(unused)]
mod utility;

pub static ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([221u8; 32]).unwrap());
pub static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
pub static ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());

pub static ACCOUNT_2_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes([212u8; 32]).unwrap());
pub static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_2_SECRET_KEY));
pub static ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_2_PUBLIC_KEY.to_account_hash());

pub const TRANSFER_AMOUNT_1: u64 = 200_001;
pub const TRANSFER_AMOUNT_2: u64 = 19_999;
pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

pub const METHOD_TRANSFER_AS_STORED_CONTRACT: &str = "transfer_as_stored_contract";
pub const METHOD_APPROVE_AS_STORED_CONTRACT: &str = "approve_as_stored_contract";
pub const METHOD_FROM_AS_STORED_CONTRACT: &str = "transfer_from_as_stored_contract";

pub const TOKEN_OWNER_ADDRESS_1: Key = Key::Account(AccountHash::new([42; 32]));
pub const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
pub const TOKEN_OWNER_ADDRESS_2: Key = Key::Hash([42; 32]);
pub const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;

pub const METHOD_MINT: &str = "mint";
pub const METHOD_BURN: &str = "burn";
pub const DECREASE_ALLOWANCE: &str = "decrease_allowance";
pub const INCREASE_ALLOWANCE: &str = "increase_allowance";
pub const ENABLE_MINT_BURN: &str = "enable_mint_burn";
pub const ADMIN_LIST: &str = "admin_list";
pub const MINTER_LIST: &str = "minter_list";
pub const NONE_LIST: &str = "none_list";
pub const CHANGE_SECURITY: &str = "change_security";

pub(crate) fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut LmdbWasmTestBuilder,
    cep18_test_contract_package: PackageHash,
) -> T {
    let contract_package = builder
        .get_package(cep18_test_contract_package)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();

    let contract_hash = enabled_versions
        .contract_hashes()
        .last()
        .expect("should have latest version");
    let entity_addr = EntityAddr::new_smart_contract(contract_hash.value());
    builder.get_value(entity_addr, RESULT_KEY)
}

pub(crate) fn cep18_check_balance_of(
    builder: &mut LmdbWasmTestBuilder,
    cep18_contract_hash: &AddressableEntityHash,
    address: Key,
) -> U256 {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let cep18_test_contract_package = account
        .named_keys()
        .get(CEP18_TEST_CONTRACT_KEY)
        .and_then(|key| key.into_package_hash())
        .expect("should have test contract hash");

    let check_balance_args = runtime_args! {
        ARG_TOKEN_CONTRACT => Key::addressable_entity_key(EntityKindTag::SmartContract, *cep18_contract_hash),
        ARG_ADDRESS => address,
    };
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_test_contract_package,
        None,
        CHECK_BALANCE_OF_ENTRYPOINT,
        check_balance_args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, cep18_test_contract_package)
}

fn generate_current_fixture() {
    let genesis_request = create_run_genesis_request(vec![
        GenesisAccount::Account {
            public_key: DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_1_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
        GenesisAccount::Account {
            public_key: ACCOUNT_2_PUBLIC_KEY.clone(),
            balance: Motes::new(U512::from(5_000_000_000_000_u64)),
            validator: None,
        },
    ]);
    generate_fixture("cep18-2.0.0-rc3-minted", genesis_request, |builder|{
        let mint_amount = U256::one();

        let install_args = runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            ENABLE_MINT_BURN => true,
        };
        let install_request_1 =
            ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CEP18_CONTRACT_WASM, install_args)
                .build();

        let install_request_2 = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            CEP18_TEST_CONTRACT_WASM,
            RuntimeArgs::default(),
        )
        .build();

        builder.exec(install_request_1).expect_success().commit();
        builder.exec(install_request_2).expect_success().commit();

        let account = builder
            .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
            .unwrap();
        let account_named_keys = account.named_keys();

        let cep18_contract_hash = account_named_keys
            .get(CEP18_TOKEN_CONTRACT_KEY)
            .and_then(|key| key.into_entity_hash())
            .expect("should have contract hash");

        let addressable_cep18_contract_hash = AddressableEntityHash::new(cep18_contract_hash.value());
        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            METHOD_MINT,
            runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
        )
        .build();
        builder.exec(mint_request).expect_success().commit();
        let mint_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            METHOD_MINT,
            runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_2, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_2)},
        )
        .build();
        builder.exec(mint_request_2).expect_success().commit();
        assert_eq!(
            cep18_check_balance_of(
                builder,
                &cep18_contract_hash,
                Key::Account(*DEFAULT_ACCOUNT_ADDR)
            ),
            U256::from(TOKEN_TOTAL_SUPPLY),
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
            U256::from(TOKEN_OWNER_AMOUNT_1)
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_2),
            U256::from(TOKEN_OWNER_AMOUNT_2)
        );

        let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            addressable_cep18_contract_hash,
            METHOD_MINT,
            runtime_args! {
                ARG_OWNER => TOKEN_OWNER_ADDRESS_1,
                ARG_AMOUNT => mint_amount,
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();

        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
            U256::from(TOKEN_OWNER_AMOUNT_1) + mint_amount,
        );
        assert_eq!(
            cep18_check_balance_of(builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_2),
            U256::from(TOKEN_OWNER_AMOUNT_2)
        );
    }).unwrap();
}

fn generate_cross_generational_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        builder_from_global_state_fixture("cep18-1.5.6-minted");

    let lmdb_fixtures_root = path_to_lmdb_fixtures();
    let fixture_root = lmdb_fixtures_root.join("cross_generation");

    let path_to_data_lmdb = fixture_root.join("global_state").join("data.lmdb");
    if path_to_data_lmdb.exists() {
        eprintln!(
            "Lmdb fixture located at {} already exists. If you need to re-generate a fixture to ensure a serialization \
            changes are backwards compatible please make sure you are running a specific version, or a past commit. \
            Skipping.",
            path_to_data_lmdb.display()
        );
        return Ok(());
    }

    assert_eq!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    // we upgrade the execution engines protocol from 1.x to 2.x
    let mut upgrade_config = UpgradeRequestBuilder::new()
        .with_current_protocol_version(lmdb_fixture_state.genesis_protocol_version())
        .with_new_protocol_version(ProtocolVersion::V2_0_0)
        .with_migrate_legacy_accounts(true)
        .with_migrate_legacy_contracts(true)
        .with_activation_point(EraId::new(1))
        .build();

    builder
        .upgrade(&mut upgrade_config)
        .expect_upgrade_success()
        .commit();

    // the state hash should now be different
    assert_ne!(
        builder.get_post_state_hash(),
        lmdb_fixture_state.post_state_hash
    );

    let post_state_hash = builder.get_post_state_hash();

    let state = LmdbFixtureState {
        genesis_request: lmdb_fixture_state.genesis_request,
        post_state_hash,
    };
    let serialized_state = serde_json::to_string_pretty(&state)?;

    let path_to_state_file = fixture_root.join(STATE_JSON_FILE);

    let mut f = File::create(path_to_state_file)?;
    f.write_all(serialized_state.as_bytes())?;
    shrink_db(path_to_data_lmdb);
    Ok(())
}

fn main() {
    generate_current_fixture();
    let _ = generate_cross_generational_fixture();
}
