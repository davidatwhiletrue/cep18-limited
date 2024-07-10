use core::convert::TryFrom;

use alloc::{collections::BTreeMap, format};
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{bytesrepr::FromBytes, contract_messages::MessagePayload, Key, U256};

use crate::{
    constants::{EVENTS, EVENTS_MODE},
    modalities::EventsMode,
    utils::{read_from, SecurityBadge},
    Cep18Error,
};

use casper_event_standard::{emit, Event, Schemas};

pub fn record_event_dictionary(event: Event) {
    let events_mode: EventsMode = EventsMode::try_from(read_from::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);

    match events_mode {
        EventsMode::NoEvents => {}
        EventsMode::CES => ces(event),
        EventsMode::Native => runtime::emit_message(EVENTS, &format!("{event:?}").into())
            .unwrap_or_revert(),
        EventsMode::NativeBytes => {
            let payload = MessagePayload::from_bytes(format!("{event:?}").as_bytes()).unwrap_or_revert().0;
            runtime::emit_message(EVENTS, &payload)
            .unwrap_or_revert()
        },
        EventsMode::NativeNCES => {
            runtime::emit_message(EVENTS, &format!("{event:?}").into())
                .unwrap_or_revert();
            ces(event);
        },
        EventsMode::NativeBytesNCES => {
            let payload = MessagePayload::from_bytes(format!("{event:?}").as_bytes()).unwrap_or_revert().0;
            runtime::emit_message(EVENTS, &payload)
            .unwrap_or_revert();
            ces(event);
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Mint(Mint),
    Burn(Burn),
    SetAllowance(SetAllowance),
    IncreaseAllowance(IncreaseAllowance),
    DecreaseAllowance(DecreaseAllowance),
    Transfer(Transfer),
    TransferFrom(TransferFrom),
    ChangeSecurity(ChangeSecurity),
    ChangeEventsMode(ChangeEventsMode),
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Mint {
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Burn {
    pub owner: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct SetAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct IncreaseAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
    pub inc_by: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct DecreaseAllowance {
    pub owner: Key,
    pub spender: Key,
    pub allowance: U256,
    pub decr_by: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub sender: Key,
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct TransferFrom {
    pub spender: Key,
    pub owner: Key,
    pub recipient: Key,
    pub amount: U256,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ChangeSecurity {
    pub admin: Key,
    pub sec_change_map: BTreeMap<Key, SecurityBadge>,
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct ChangeEventsMode {
    pub events_mode: u8,
}

fn ces(event: Event) {
    match event {
        Event::Mint(ev) => emit(ev),
        Event::Burn(ev) => emit(ev),
        Event::SetAllowance(ev) => emit(ev),
        Event::IncreaseAllowance(ev) => emit(ev),
        Event::DecreaseAllowance(ev) => emit(ev),
        Event::Transfer(ev) => emit(ev),
        Event::TransferFrom(ev) => emit(ev),
        Event::ChangeSecurity(ev) => emit(ev),
        Event::ChangeEventsMode(ev) => emit(ev),
    }
}

pub fn init_events() {
    let events_mode: EventsMode = EventsMode::try_from(read_from::<u8>(EVENTS_MODE))
        .unwrap_or_revert_with(Cep18Error::InvalidEventsMode);

    if [EventsMode::CES, EventsMode::NativeNCES, EventsMode::NativeBytesNCES].contains(&events_mode)
        && runtime::get_key(casper_event_standard::EVENTS_DICT).is_none()
    {
        let schemas = Schemas::new()
            .with::<Mint>()
            .with::<Burn>()
            .with::<SetAllowance>()
            .with::<IncreaseAllowance>()
            .with::<DecreaseAllowance>()
            .with::<Transfer>()
            .with::<TransferFrom>()
            .with::<ChangeSecurity>()
            .with::<ChangeEventsMode>();
        casper_event_standard::init(schemas);
    }
}
