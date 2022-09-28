#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use osmo_bindings::OsmosisMsg;

use crate::error::ContractError;
use crate::execute;
use crate::hooks;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::queries;
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tokenfactory-issuer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<OsmosisMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        is_frozen: false,
        denom: msg.denom.clone(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("denom", msg.denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<OsmosisMsg>, ContractError> {
    match msg {
        // Executive Functions
        ExecuteMsg::Mint { to_address, amount } => {
            execute::mint(deps, env, info, to_address, amount)
        }
        ExecuteMsg::Burn { amount } => execute::burn(deps, info, amount),
        ExecuteMsg::Blacklist { address, status } => {
            execute::blacklist(deps, info, address, status)
        }
        ExecuteMsg::Freeze { status } => execute::freeze(deps, info, status),

        // Admin functions
        ExecuteMsg::ChangeTokenFactoryAdmin { new_admin } => {
            execute::change_tokenfactory_admin(deps, info, new_admin)
        }
        ExecuteMsg::ChangeContractOwner { new_owner } => {
            execute::change_contract_owner(deps, info, new_owner)
        }
        ExecuteMsg::SetMinter { address, allowance } => {
            execute::set_minter(deps, info, address, allowance)
        }
        ExecuteMsg::SetBurner { address, allowance } => {
            execute::set_burner(deps, info, address, allowance)
        }
        ExecuteMsg::SetBlacklister { address, status } => {
            execute::set_blacklister(deps, info, address, status)
        }
        ExecuteMsg::SetFreezer { address, status } => {
            execute::set_freezer(deps, info, address, status)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::BlockBeforeSend { from, to, amount } => hooks::blockbeforesend_hook(deps, from, to, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsFrozen {} => to_binary(&queries::query_is_frozen(deps)?),
        QueryMsg::Denom {} => to_binary(&queries::query_denom(deps)?),
        QueryMsg::Owner {} => to_binary(&queries::query_owner(deps)?),
        QueryMsg::BurnAllowance { address } => {
            to_binary(&queries::query_burn_allowance(deps, address)?)
        }
        QueryMsg::BurnAllowances { start_after, limit } => {
            to_binary(&queries::query_burn_allowances(deps, start_after, limit)?)
        }
        QueryMsg::MintAllowance { address } => {
            to_binary(&queries::query_mint_allowance(deps, address)?)
        }
        QueryMsg::MintAllowances { start_after, limit } => {
            to_binary(&queries::query_mint_allowances(deps, start_after, limit)?)
        }
        QueryMsg::IsBlacklisted { address } => {
            to_binary(&queries::query_is_blacklisted(deps, address)?)
        }
        QueryMsg::Blacklistees { start_after, limit } => {
            to_binary(&queries::query_blacklistees(deps, start_after, limit)?)
        }
        QueryMsg::IsBlacklister { address } => {
            to_binary(&queries::query_is_blacklister(deps, address)?)
        }
        QueryMsg::BlacklisterAllowances { start_after, limit } => to_binary(
            &queries::query_blacklister_allowances(deps, start_after, limit)?,
        ),
        QueryMsg::IsFreezer { address } => to_binary(&queries::query_is_freezer(deps, address)?),
        QueryMsg::FreezerAllowances { start_after, limit } => to_binary(
            &queries::query_freezer_allowances(deps, start_after, limit)?,
        ),
    }
}
