use crate::steak::LsdContractsRaw;

use crate::steak::SteakLSDHub;
use crate::testing::mock_querier::{WasmMockQuerier, MOCK_HUB_CONTRACT_ADDR};
use basset::external::LSDStateResponse;
use basset::wrapper::ExecuteMsg;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, to_binary, Api, CosmosMsg, Decimal, DepsMut, OwnedDeps, Storage, SubMsg, Uint128,
    WasmMsg,
};
use std::str::FromStr;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};
use cw20_base::contract::{query_minter, query_token_info};

use crate::testing::mock_querier::{
    mock_dependencies, MOCK_LSD_HUB_CONTRACT_ADDR, MOCK_LSD_TOKEN_CONTRACT_ADDR,
};
use cavern_lsd_wrapper_token::contract::{execute, instantiate};
use cavern_lsd_wrapper_token::msg::TokenInitMsg;
use std::borrow::BorrowMut;

// this will set up the init for other tests
fn do_init_with_minter<S: Storage, A: Api>(
    deps: &mut OwnedDeps<S, A, WasmMockQuerier>,
    minter: &str,
    cap: Option<Uint128>,
) -> TokenInfoResponse {
    _do_init(
        deps,
        Some(MinterResponse {
            minter: minter.into(),
            cap,
        }),
    )
}

// this will set up the init for other tests
fn _do_init<S: Storage, A: Api>(
    deps: &mut OwnedDeps<S, A, WasmMockQuerier>,
    mint: Option<MinterResponse>,
) -> TokenInfoResponse {
    let lsd_hub_contract = String::from(MOCK_LSD_HUB_CONTRACT_ADDR);
    let lsd_token_contract = String::from(MOCK_LSD_TOKEN_CONTRACT_ADDR);

    let lsd_config = LsdContractsRaw {
        token: lsd_token_contract,
        hub: lsd_hub_contract,
    };

    let init_msg = TokenInitMsg {
        types: None,
        name: "bluna".to_string(),
        symbol: "BLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        hub_contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
        lsd_config,
    };

    let info = mock_info(&String::from("owner"), &[]);
    let res = instantiate::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        init_msg,
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "bluna".to_string(),
            symbol: "BLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );
    assert_eq!(query_minter(deps.as_ref()).unwrap(), mint,);

    // We setup the LSD with an initial exchange rate...

    deps.querier.with_lsd_state(LSDStateResponse {
        exchange_rate: Decimal::from_str("1.5").unwrap(),
        total_usteak: Uint128::from(1000000000u128),
        total_uluna: Uint128::from(1500000000u128),
        unlocked_coins: vec![],
    });

    meta
}

pub fn do_mint(deps: DepsMut, addr: String, amount: Uint128, exchange_rate: Decimal) {
    let msg = ExecuteMsg::Mint {
        recipient: addr,
        amount,
    };
    let minter = "any_person_really";
    let info = mock_info(minter, &[]);
    let res =
        execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(deps, mock_env(), info, msg)
            .unwrap();
    assert_eq!(1, res.messages.len());

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_LSD_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                recipient: MOCK_CONTRACT_ADDR.to_string(),
                owner: minter.to_string(),
                amount: Decimal::from_ratio(amount, 1u128) / exchange_rate * Uint128::one()
                    + Uint128::one()
            })
            .unwrap(),
            funds: vec![]
        }))]
    );
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let lsd_hub_contract = String::from(MOCK_LSD_HUB_CONTRACT_ADDR);
    let lsd_token_contract = String::from(MOCK_LSD_TOKEN_CONTRACT_ADDR);

    let lsd_config = LsdContractsRaw {
        token: lsd_token_contract,
        hub: lsd_hub_contract,
    };

    let init_msg = TokenInitMsg {
        types: None,
        name: "bluna".to_string(),
        symbol: "BLUNA".to_string(),
        decimals: 6,
        initial_balances: vec![],
        hub_contract: MOCK_HUB_CONTRACT_ADDR.to_string(),
        lsd_config,
    };
    let info = mock_info(&String::from("owner"), &[]);
    let res = instantiate::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        init_msg,
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "bluna".to_string(),
            symbol: "BLUNA".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
        }
    );

    assert_eq!(
        query_minter(deps.as_ref()).unwrap(),
        Some(MinterResponse {
            minter: MOCK_CONTRACT_ADDR.to_string(),
            cap: None
        })
    );
}

#[test]
fn transfer() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let info = mock_info(addr1.as_str(), &[]);
    let msg = ExecuteMsg::Transfer {
        recipient: addr2,
        amount: Uint128::new(1u128),
    };

    let _res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();
}

#[test]
fn transfer_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let addr3 = String::from("addr0003");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let info = mock_info(addr1.as_str(), &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr3.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    let info = mock_info(addr3.as_str(), &[]);
    let msg = ExecuteMsg::TransferFrom {
        owner: addr1,
        recipient: addr2,
        amount: Uint128::new(1u128),
    };

    let _res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();
}

#[test]
fn mint() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = String::from("addr0000");

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);

    let info = mock_info(&String::from("owner"), &[]);
    let msg = ExecuteMsg::Mint {
        recipient: addr,
        amount: Uint128::new(1u128),
    };

    let _res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();
}

#[test]
fn mint_multiple_exchange_rates() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = String::from("addr0000");

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);

    let info = mock_info(&String::from("owner"), &[]);
    let msg = ExecuteMsg::Mint {
        recipient: addr.clone(),
        amount: Uint128::new(1u128),
    };

    let _res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    // Now, we mint again with the same exchange rate by checking that it minted the right amount of tokens

    do_mint(
        deps.as_mut(),
        addr.clone(),
        Uint128::from(10u128),
        Decimal::from_str("1.5").unwrap(),
    );
    deps.querier.with_lsd_state(LSDStateResponse {
        exchange_rate: Decimal::from_str("1").unwrap(),
        total_usteak: Uint128::from(1000000000u128),
        total_uluna: Uint128::from(1500000000u128),
        unlocked_coins: vec![],
    });
    do_mint(
        deps.as_mut(),
        addr.clone(),
        Uint128::from(10u128),
        Decimal::from_str("1").unwrap(),
    );

    deps.querier.with_lsd_state(LSDStateResponse {
        exchange_rate: Decimal::from_str("0.1").unwrap(),
        total_usteak: Uint128::from(1000000000u128),
        total_uluna: Uint128::from(1500000000u128),
        unlocked_coins: vec![],
    });
    do_mint(
        deps.as_mut(),
        addr,
        Uint128::from(10u128),
        Decimal::from_str("0.1").unwrap(),
    );
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = String::from("addr0000");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let info = mock_info(addr.as_str(), &[]);
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(1234u128),
    };

    let res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    // When you burn, you should get your lsd token back
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_LSD_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: addr,
                amount: Uint128::new(822u128)
            })
            .unwrap(),
            funds: vec![],
        }))]
    )
}

#[test]
fn burn_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr = String::from("addr0000");
    let addr1 = String::from("addr0001");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let info = mock_info(addr.as_str(), &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr1.clone(),
        amount: Uint128::new(1234u128),
        expires: None,
    };
    let _ = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    let info = mock_info(addr1.as_str(), &[]);
    let msg = ExecuteMsg::BurnFrom {
        owner: addr,
        amount: Uint128::new(1234u128),
    };

    let res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    // When you burn, you should get your lsd token back
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_LSD_TOKEN_CONTRACT_ADDR.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: addr1,
                amount: Uint128::new(822u128)
            })
            .unwrap(),
            funds: vec![],
        }))]
    )
}

#[test]
fn send() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let dummny_contract_addr = String::from("dummy");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(addr1.as_str(), &[]);
    let msg = ExecuteMsg::Send {
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        Cw20ReceiveMsg {
            sender: addr1,
            amount: Uint128::new(1),
            msg: to_binary(&dummy_msg).unwrap(),
        }
        .into_cosmos_msg(dummny_contract_addr)
        .unwrap()
    );
}

#[test]
fn send_from() {
    let mut deps = mock_dependencies(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let dummny_contract_addr = String::from("dummy");
    let amount1 = Uint128::from(12340000u128);

    do_init_with_minter(deps.borrow_mut(), &String::from(MOCK_CONTRACT_ADDR), None);
    do_mint(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        Decimal::from_str("1.5").unwrap(),
    );

    let info = mock_info(addr1.as_str(), &[]);
    let msg = ExecuteMsg::IncreaseAllowance {
        spender: addr2.clone(),
        amount: Uint128::new(1u128),
        expires: None,
    };
    let _ = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();

    let dummy_msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1u128),
    };

    let info = mock_info(addr2.as_str(), &[]);
    let msg = ExecuteMsg::SendFrom {
        owner: addr1,
        contract: dummny_contract_addr.clone(),
        amount: Uint128::new(1u128),
        msg: to_binary(&dummy_msg).unwrap(),
    };

    let res = execute::<LsdContractsRaw, SteakLSDHub<LSDStateResponse>>(
        deps.as_mut(),
        mock_env(),
        info,
        msg,
    )
    .unwrap();
    assert_eq!(res.messages.len(), 1);

    assert_eq!(
        res.messages[0].msg,
        Cw20ReceiveMsg {
            sender: addr2,
            amount: Uint128::new(1),
            msg: to_binary(&dummy_msg).unwrap(),
        }
        .into_cosmos_msg(dummny_contract_addr)
        .unwrap()
    );
}
