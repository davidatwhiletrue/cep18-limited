//! Implementation of allowances.
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{bytesrepr::ToBytes, Key, URef, U256};

use crate::{
    constants::ALLOWANCES,
    events::{record_event_dictionary, Event},
    utils, Cep18Error,
};

#[inline]
pub(crate) fn get_allowances_uref() -> URef {
    utils::get_uref(ALLOWANCES)
}

/// Creates a dictionary item key for an (owner, spender) pair.
pub(crate) fn make_dictionary_item_key(owner: Key, spender: Key) -> String {
    let mut preimage = Vec::new();
    preimage.append(
        &mut owner
            .to_bytes()
            .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
    );
    preimage.append(
        &mut spender
            .to_bytes()
            .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
    );

    let key_bytes = runtime::blake2b(&preimage);
    let msg = format!("XXX key_bytes {:?}", key_bytes);
    record_event_dictionary(Event::RawMsg(msg.to_string()));
    hex::encode(key_bytes)
}

/// Writes an allowance for owner and spender for a specific amount.
pub(crate) fn write_allowance_to(allowance_uref: URef, owner: Key, spender: Key, amount: U256) {
    let dictionary_item_key = make_dictionary_item_key(owner, spender);
    let msg = format!("XXX dictionary_item_key {:?}", dictionary_item_key);
    record_event_dictionary(Event::RawMsg(msg.to_string()));
    storage::dictionary_put(allowance_uref, &dictionary_item_key, amount)
}

/// Reads an allowance for a owner and spender
pub(crate) fn read_allowance_from(allowances_uref: URef, owner: Key, spender: Key) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(owner, spender);
    storage::dictionary_get(allowances_uref, &dictionary_item_key)
        .unwrap_or_revert_with(Cep18Error::FailedToGetDictionaryValue)
        .unwrap_or_default()
}
