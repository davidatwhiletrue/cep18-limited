use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, UpgradeRequestBuilder, DEFAULT_ACCOUNT_ADDR,
};
use casper_fixtures::LmdbFixtureState;
use casper_types::{
    bytesrepr::FromBytes, runtime_args, AddressableEntityHash, CLTyped, EraId, Key,
    ProtocolVersion, RuntimeArgs, U256,
};

use crate::utility::{
    constants::{
        AMOUNT, ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, CEP18_CONTRACT_WASM,
        CEP18_TEST_CONTRACT_WASM, CEP18_TOKEN_CONTRACT_KEY, CEP18_TOKEN_CONTRACT_VERSION_KEY,
        EVENTS, EVENTS_MODE, METHOD_MINT, MIGRATE_USER_BALANCE_KEYS_ENTRY_POINT_NAME,
        MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME, OWNER, TOKEN_DECIMALS, TOKEN_NAME,
        TOKEN_OWNER_ADDRESS_1, TOKEN_OWNER_ADDRESS_1_OLD, TOKEN_OWNER_AMOUNT_1, TOKEN_SYMBOL,
        TOKEN_TOTAL_SUPPLY, USER_KEY_MAP,
    },
    installer_request_builders::cep18_check_balance_of,
    message_handlers::{entity, message_summary, message_topic},
};

pub fn upgrade_v1_5_6_fixture_to_v2_0_0_ee(
    builder: &mut LmdbWasmTestBuilder,
    lmdb_fixture_state: &LmdbFixtureState,
) {
    // state hash in builder and lmdb storage should be the same
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
}

pub fn query_contract_value<T: CLTyped + FromBytes>(
    builder: &LmdbWasmTestBuilder,
    path: &[String],
) -> T {
    builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), path)
        .unwrap()
        .as_cl_value()
        .unwrap()
        .clone()
        .into_t()
        .unwrap()
}

// the difference between the two is that in v1_binary the contract hash is fetched at [u8;32], while in v2_binary it is an AddressaleEntityHash
pub fn get_contract_hash_v1_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_token = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_hash_addr())
        .map(AddressableEntityHash::new)
        .expect("should have contract hash");

    cep18_token
}

pub fn get_contract_hash_v2_binary(builder: &LmdbWasmTestBuilder) -> AddressableEntityHash {
    let account = builder
        .get_entity_with_named_keys_by_account_hash(*DEFAULT_ACCOUNT_ADDR)
        .unwrap();
    let account_named_keys = account.named_keys();

    let cep18_token = account_named_keys
        .get(CEP18_TOKEN_CONTRACT_KEY)
        .and_then(|key| key.into_entity_hash())
        .expect("should have contract hash");

    cep18_token
}

#[test]
fn should_be_able_to_call_1x_contract_in_2x_execution_engine() {
    // load fixture that was created in a previous EE version
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep18-1.5.6-minted");

    // upgrade the execution engine to the new protocol version
    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let cep18_token = get_contract_hash_v1_binary(&builder);

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1_OLD, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();
}

#[test]
fn should_migrate_1_5_6_to_2_0_0_rc3() {
    // load fixture
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep18-1.5.6-minted");

    // upgrade engine
    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let version_0: u32 =
        query_contract_value(&builder, &[CEP18_TOKEN_CONTRACT_VERSION_KEY.to_string()]);

    // upgrade the contract itself using a binary built for the new engine
    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            EVENTS_MODE => 2_u8,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let version_1: u32 =
        query_contract_value(&builder, &[CEP18_TOKEN_CONTRACT_VERSION_KEY.to_string()]);

    assert!(version_0 < version_1);

    let cep18_contract_hash = get_contract_hash_v2_binary(&builder);

    // use utility entrypoint to migrate the security keys in cep-18
    let mut user_map: Vec<Key> = Vec::new();
    user_map.push(Key::Account(*DEFAULT_ACCOUNT_ADDR));
    user_map.push(TOKEN_OWNER_ADDRESS_1_OLD);

    let sec_key_migrate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME,
        runtime_args! {USER_KEY_MAP => &user_map},
    )
    .build();

    builder
        .exec(sec_key_migrate_request)
        .expect_success()
        .commit();

    // mint some new tokens in cep-18
    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    let test_contract = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_TEST_CONTRACT_WASM,
        RuntimeArgs::default(),
    )
    .build();

    builder.exec(test_contract).expect_success().commit();

    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
        U256::from(TOKEN_OWNER_AMOUNT_1),
    );

    // migrate the balances from the old owner keys to the new ones in cep-18
    let balance_migrate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_contract_hash,
        MIGRATE_USER_BALANCE_KEYS_ENTRY_POINT_NAME,
        runtime_args! {USER_KEY_MAP => user_map},
    )
    .build();

    builder
        .exec(balance_migrate_request)
        .expect_success()
        .commit();

    // even if we minted before migrating, the balance should persist
    assert_eq!(
        cep18_check_balance_of(&mut builder, &cep18_contract_hash, TOKEN_OWNER_ADDRESS_1),
        U256::from(TOKEN_OWNER_AMOUNT_1 * 2),
    );
}

#[test]
fn should_have_native_events() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        casper_fixtures::builder_from_global_state_fixture("cep18-1.5.6-minted");

    upgrade_v1_5_6_fixture_to_v2_0_0_ee(&mut builder, &lmdb_fixture_state);

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP18_CONTRACT_WASM,
        runtime_args! {
            ARG_NAME => TOKEN_NAME,
            ARG_SYMBOL => TOKEN_SYMBOL,
            ARG_DECIMALS => TOKEN_DECIMALS,
            ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
            EVENTS_MODE => 3_u8,
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    let cep18_token = get_contract_hash_v2_binary(&builder);

    // events check
    let entity = entity(&builder, &cep18_token);

    let (topic_name, message_topic_hash) = entity
        .message_topics()
        .iter()
        .last()
        .expect("should have at least one topic");

    let mut user_map: Vec<Key> = Vec::new();
    user_map.push(Key::Account(*DEFAULT_ACCOUNT_ADDR));
    user_map.push(TOKEN_OWNER_ADDRESS_1_OLD);

    let sec_key_migrate_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        MIGRATE_USER_SEC_KEYS_ENTRY_POINT_NAME,
        runtime_args! {USER_KEY_MAP => &user_map},
    )
    .build();

    builder
        .exec(sec_key_migrate_request)
        .expect_success()
        .commit();

    assert_eq!(topic_name, &EVENTS.to_string());

    let mint_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        cep18_token,
        METHOD_MINT,
        runtime_args! {OWNER => TOKEN_OWNER_ADDRESS_1, AMOUNT => U256::from(TOKEN_OWNER_AMOUNT_1)},
    )
    .build();

    builder.exec(mint_request).expect_success().commit();

    assert_eq!(
        message_topic(&builder, &cep18_token, *message_topic_hash).message_count(),
        2
    );

    message_summary(&builder, &cep18_token, message_topic_hash, 0, None).unwrap();
}
