#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Order, Reply, Response,
    StdResult, SubMsg, SubMsgResponse, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::parse_instantiate_response_data;

use crate::error::ContractError;
use crate::msg::{
    AccountResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ListValidatorsResponse, QueryMsg,
    StakeInfo, ValidatorResponse,
};
use crate::state::{Config, ValStatus, Validator, CONFIG, STAKED, VALIDATORS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:mesh-provider";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// for reply callbacks
const INIT_CALLBACK_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        consumer: msg.consumer,
        slasher: None,
        lockup: deps.api.addr_validate(&msg.lockup)?,
        unbonding_period: msg.unbonding_period,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

    let label = format!("Slasher for {}", &env.contract.address);
    let msg = WasmMsg::Instantiate {
        admin: Some(env.contract.address.into_string()),
        code_id: msg.slasher.code_id,
        msg: msg.slasher.msg,
        funds: vec![],
        label,
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INIT_CALLBACK_ID => reply_init_callback(deps, reply.result.unwrap()),
        _ => Err(ContractError::InvalidReplyId(reply.id)),
    }
}

pub fn reply_init_callback(deps: DepsMut, resp: SubMsgResponse) -> Result<Response, ContractError> {
    CONFIG.update::<_, ContractError>(deps.storage, |mut cfg| {
        let init_response = parse_instantiate_response_data(&resp.data.unwrap_or_default())?;
        cfg.slasher = Some(deps.api.addr_validate(&init_response.contract_address)?);
        Ok(cfg)
    })?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveClaim {
            owner,
            amount,
            validator,
        } => execute_receive_claim(deps, info, owner, amount, validator),
        ExecuteMsg::Slash {
            validator,
            percentage,
            force_unbond,
        } => execute_slash(deps, info, validator, percentage, force_unbond),
        ExecuteMsg::Unstake { amount, validator } => execute_unstake(deps, info, validator, amount),
        ExecuteMsg::Unbond {} => execute_unbond(deps, info),
    }
}

pub fn execute_receive_claim(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
    validator: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(cfg.lockup, info.sender, ContractError::Unauthorized);
    let owner = deps.api.addr_validate(&owner)?;

    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
    let mut stake = STAKED
        .may_load(deps.storage, (&owner, &validator))?
        .unwrap_or_default();
    stake.stake_validator(&mut val, amount);
    STAKED.save(deps.storage, (&owner, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // TODO: send out IBC packet for additional power (validator, amount)

    Ok(Response::new())
}

pub fn execute_slash(
    deps: DepsMut,
    info: MessageInfo,
    validator: String,
    percentage: Decimal,
    force_unbond: bool,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    ensure_eq!(cfg.slasher, Some(info.sender), ContractError::Unauthorized);
    if percentage.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    VALIDATORS.update::<_, ContractError>(deps.storage, &validator, |val| {
        let mut val = val.ok_or(ContractError::UnknownValidator(validator.clone()))?;
        val.slash(percentage);
        if force_unbond {
            val.status = ValStatus::Tombstoned;
        }
        Ok(val)
    })?;

    Ok(Response::new()
        .add_attribute("action", "slash")
        .add_attribute("validator", validator))
}

pub fn execute_unstake(
    deps: DepsMut,
    info: MessageInfo,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount);
    }

    // updates the stake
    let mut val = VALIDATORS
        .may_load(deps.storage, &validator)?
        .ok_or_else(|| ContractError::UnknownValidator(validator.clone()))?;
    if val.status != ValStatus::Active {
        return Err(ContractError::RemovedValidator(validator));
    }
    let mut stake = STAKED.load(deps.storage, (&info.sender, &validator))?;
    stake.unstake_validator(&mut val, amount)?;
    STAKED.save(deps.storage, (&info.sender, &validator), &stake)?;
    VALIDATORS.save(deps.storage, &validator, &val)?;

    // TODO: create a future claim

    // TODO: send IBC packet on change of stake

    Ok(Response::new())
}

pub fn execute_unbond(_deps: DepsMut, _info: MessageInfo) -> Result<Response, ContractError> {
    // TODO
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Account { address } => to_binary(&query_account(deps, address)?),
        QueryMsg::Validator { address } => to_binary(&query_validator(deps, address)?),
        QueryMsg::ListValidators { start_after, limit } => {
            to_binary(&list_validators(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        consumer: cfg.consumer,
        slasher: cfg.slasher.map(|x| x.into_string()),
    })
}

pub fn query_account(deps: Deps, address: String) -> StdResult<AccountResponse> {
    let account = deps.api.addr_validate(&address)?;
    let staked = STAKED
        .prefix(&account)
        .range(deps.storage, None, None, Order::Ascending)
        .map(|res| {
            let (validator, stake) = res?;
            let val = VALIDATORS.load(deps.storage, &validator)?;
            let tokens = stake.current_value(&val);
            let slashed = stake.locked - tokens;
            Ok(StakeInfo {
                validator,
                tokens,
                slashed,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;
    Ok(AccountResponse { staked })
}

pub fn query_validator(deps: Deps, address: String) -> StdResult<ValidatorResponse> {
    let val = VALIDATORS.load(deps.storage, &address)?;
    Ok(build_response((address, val)))
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 30;

pub fn list_validators(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListValidatorsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|x| Bound::exclusive(x.as_str()));

    let validators = VALIDATORS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|r| Ok(build_response(r?)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListValidatorsResponse { validators })
}

fn build_response((address, val): (String, Validator)) -> ValidatorResponse {
    ValidatorResponse {
        address,
        tokens: val.stake_value(),
        status: val.status,
        multiplier: val.multiplier,
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ConsumerInfo, SlasherInfo};

    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            consumer: ConsumerInfo {
                connection_id: "1".to_string(),
            },
            slasher: SlasherInfo {
                code_id: 17,
                msg: b"{}".into(),
            },
            lockup: "lockup_contract".to_string(),
            unbonding_period: 86400 * 14,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
