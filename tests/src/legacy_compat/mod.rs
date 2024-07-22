use casper_engine_test_support::LmdbWasmTestBuilder;
use casper_types::{runtime_args, U256};

use crate::utility::{
    constants::{
        ARG_DECIMALS, ARG_NAME, ARG_SYMBOL, ARG_TOTAL_SUPPLY, EVENTS_MODE, LEGACY_KEY_COMPAT,
        TOKEN_DECIMALS, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY,
    },
    installer_request_builders::{setup_with_args, TestContext},
};

#[cfg(test)]
mod allowance;
#[cfg(test)]
mod events;
#[cfg(test)]
mod install;
#[cfg(test)]
mod migration;
#[cfg(test)]
mod mint_and_burn;
#[cfg(test)]
mod transfer;
#[cfg(test)]
mod upgrade;

pub(crate) fn setup() -> (LmdbWasmTestBuilder, TestContext) {
    setup_with_args(runtime_args! {
        ARG_NAME => TOKEN_NAME,
        ARG_SYMBOL => TOKEN_SYMBOL,
        ARG_DECIMALS => TOKEN_DECIMALS,
        ARG_TOTAL_SUPPLY => U256::from(TOKEN_TOTAL_SUPPLY),
        EVENTS_MODE => 2_u8,
        LEGACY_KEY_COMPAT => 1_u8
    })
}
