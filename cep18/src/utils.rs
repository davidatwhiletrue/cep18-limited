//! Implementation details.
use core::mem::MaybeUninit;

use alloc::{collections::BTreeMap, vec, vec::Vec};
use casper_contract::{
    contract_api::{
        self,
        runtime::{self, revert},
        storage::{self, dictionary_get, dictionary_put},
    },
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash,
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    system::{CallStackElement, Caller},
    ApiError, CLTyped, EntityAddr, Key, URef, U256,
};

use crate::{
    constants::{LEGACY_KEY_COMPAT, SECURITY_BADGES, TOTAL_SUPPLY},
    error::Cep18Error,
    modalities::LegacyKeyCompat,
};

/// Gets [`URef`] under a name.
pub(crate) fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(Cep18Error::FailedToGetKey)
        .unwrap_or_revert();
    *key.as_uref()
        .unwrap_or_revert_with(Cep18Error::InvalidKeyType)
}

/// Reads value from a named key.
pub(crate) fn read_from<T>(name: &str) -> T
where
    T: FromBytes + CLTyped,
{
    let uref = get_uref(name);
    let value: T = storage::read(uref)
        .unwrap_or_revert_with(Cep18Error::UrefNotFound)
        .unwrap_or_revert_with(Cep18Error::FailedToReadFromStorage);
    value
}

fn call_stack_element_to_address(call_stack_element: Caller) -> Key {
    match call_stack_element {
        Caller::Initiator { account_hash } => {
            Key::AddressableEntity(EntityAddr::Account(account_hash.value()))
        }
        Caller::Entity { package_hash, .. } => Key::Package(package_hash.value()),
    }
}

/// Returns address based on a [`CallStackElement`].
///
/// For `Session` and `StoredSession` variants it will return account hash, and for `StoredContract`
/// case it will use contract package hash as the address.
fn call_stack_element_to_address_legacy(call_stack_element: CallStackElement) -> Key {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Key::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session wants to interact
            // with an CEP-18 token caller's address will be used.
            Key::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Key::from(contract_package_hash),
    }
}

/// Returns the call stack.
pub fn get_call_stack() -> Vec<CallStackElement> {
    let (call_stack_len, result_size) = {
        let mut call_stack_len: usize = 0;
        let mut result_size: usize = 0;
        let ret = unsafe {
            ext_ffi::casper_load_call_stack(
                &mut call_stack_len as *mut usize,
                &mut result_size as *mut usize,
            )
        };
        api_error::result_from(ret).unwrap_or_revert();
        (call_stack_len, result_size)
    };
    if call_stack_len == 0 {
        return Vec::new();
    }
    let bytes = read_host_buffer(result_size).unwrap_or_revert();
    bytesrepr::deserialize(bytes).unwrap_or_revert()
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

pub(crate) fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

pub(crate) fn get_immediate_caller_address() -> Result<Key, Cep18Error> {
    let call_stack = runtime::get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_address)
        .ok_or(Cep18Error::InvalidContext)
}

/// Gets the immediate session caller of the current execution.
///
/// This function ensures that Contracts can participate and no middleman (contract) acts for
/// users.
pub(crate) fn get_immediate_caller_address_legacy() -> Result<Key, Cep18Error> {
    let call_stack = get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_address_legacy)
        .ok_or(Cep18Error::InvalidContext)
}

pub(crate) fn get_immediate_caller_key() -> Result<Key, Cep18Error> {
    match LegacyKeyCompat::try_from(read_from::<u8>(LEGACY_KEY_COMPAT)).unwrap_or_revert() {
        LegacyKeyCompat::Condor => get_immediate_caller_address(),
        LegacyKeyCompat::Legacy => get_immediate_caller_address_legacy(),
    }
}

pub fn get_total_supply_uref() -> URef {
    get_uref(TOTAL_SUPPLY)
}

pub(crate) fn read_total_supply_from(uref: URef) -> U256 {
    storage::read(uref)
        .unwrap_or_revert_with(Cep18Error::UrefNotFound)
        .unwrap_or_revert_with(Cep18Error::FailedToReadFromStorage)
}

/// Writes a total supply to a specific [`URef`].
pub(crate) fn write_total_supply_to(uref: URef, value: U256) {
    storage::write(uref, value);
}

pub fn get_named_arg_size(name: &str) -> Option<usize> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    match api_error::result_from(ret) {
        Ok(_) => Some(arg_size),
        Err(ApiError::MissingArgument) => None,
        Err(e) => {
            runtime::revert(e)
        }
    }
}

pub fn get_optional_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    invalid: Cep18Error,
) -> Option<T> {
    match get_named_arg_with_user_errors::<T>(name, Cep18Error::Phantom, invalid) {
        Ok(val) => Some(val),
        Err(Cep18Error::Phantom) => None,
        Err(_) => runtime::revert(invalid),
    }
}

pub fn get_named_arg_with_user_errors<T: FromBytes>(
    name: &str,
    missing: Cep18Error,
    invalid: Cep18Error,
) -> Result<T, Cep18Error> {
    let arg_size = get_named_arg_size(name).ok_or(missing)?;
    let arg_bytes = if arg_size > 0 {
        let res = {
            let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
            let ret = unsafe {
                ext_ffi::casper_get_named_arg(
                    name.as_bytes().as_ptr(),
                    name.len(),
                    data_non_null_ptr.as_ptr(),
                    arg_size,
                )
            };
            let data =
                unsafe { Vec::from_raw_parts(data_non_null_ptr.as_ptr(), arg_size, arg_size) };
            api_error::result_from(ret).map(|_| data)
        };
        // Assumed to be safe as `get_named_arg_size` checks the argument already
        res.map_err(|_err| Cep18Error::FailedToGetArgBytes)
        .unwrap_or_revert()
    } else {
        // Avoids allocation with 0 bytes and a call to get_named_arg
        Vec::new()
    };

    bytesrepr::deserialize(arg_bytes).map_err(|_err| invalid)
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SecurityBadge {
    Admin = 0,
    Minter = 1,
    None = 2,
}

impl CLTyped for SecurityBadge {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::U8
    }
}

impl ToBytes for SecurityBadge {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        1
    }
}

impl FromBytes for SecurityBadge {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        Ok((
            match bytes[0] {
                0 => SecurityBadge::Admin,
                1 => SecurityBadge::Minter,
                2 => SecurityBadge::None,
                _ => return Err(bytesrepr::Error::LeftOverBytes),
            },
            &[],
        ))
    }
}

pub fn sec_check(allowed_badge_list: Vec<SecurityBadge>) {
    let caller = get_immediate_caller_key()
        .unwrap_or_revert_with(Cep18Error::FailedToRetrieveImmediateCaller);
    let uref = get_uref(SECURITY_BADGES);
    if let Some(badge) =
        dictionary_get::<SecurityBadge>(uref, &base64::encode(caller.to_bytes().unwrap_or_revert()))
            .unwrap_or_revert_with(Cep18Error::FailedToGetDictionaryValue)
    {
        if !allowed_badge_list.contains(&badge) {
            revert(Cep18Error::InsufficientRights)
        }
    } else {
        let converted_key = match caller {
            Key::Account(account) => Key::AddressableEntity(EntityAddr::Account(account.value())),
            Key::Hash(hash) => Key::Package(hash),
            Key::Package(package_hash) => Key::Hash(package_hash),
            Key::AddressableEntity(EntityAddr::Account(account)) => {
                Key::Account(AccountHash(account))
            }
            _ => revert(Cep18Error::InvalidKeyType),
        };

        if let Some(badge) = dictionary_get::<SecurityBadge>(
            uref,
            &base64::encode(converted_key.to_bytes().unwrap_or_revert()),
        )
        .unwrap_or_revert_with(Cep18Error::FailedToGetDictionaryValue)
        {
            if !allowed_badge_list.contains(&badge) {
                revert(Cep18Error::InsufficientRights)
            }
        } else {
            revert(Cep18Error::InsufficientRights)
        }
    }
}

pub fn change_sec_badge(badge_map: &BTreeMap<Key, SecurityBadge>) {
    let sec_uref = get_uref(SECURITY_BADGES);
    for (&user, &badge) in badge_map {
        dictionary_put(
            sec_uref,
            &base64::encode(
                user.to_bytes()
                    .unwrap_or_revert_with(Cep18Error::FailedToConvertBytes),
            ),
            badge,
        )
    }
}
