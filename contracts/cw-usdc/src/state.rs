use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub denom: String,
    pub owner: Addr,
    pub is_frozen: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const BLACKLISTED_ADDRESSES: Map<&Addr, bool> = Map::new("blacklisted_addresses");

pub const MINTER_ALLOWANCES: Map<&Addr, Uint128> = Map::new("minter_allowances");
pub const BURNER_ALLOWANCES: Map<&Addr, Uint128> = Map::new("burner_allowances");
pub const BLACKLISTER_ALLOWANCES: Map<&Addr, bool> = Map::new("blacklister_allowances");
pub const FREEZER_ALLOWANCES: Map<&Addr, bool> = Map::new("freezer_allowances");
pub const SUPPLY_OFFSET_ADDRESSES: Map<&Addr, bool> = Map::new("supply_offset_addresses");
