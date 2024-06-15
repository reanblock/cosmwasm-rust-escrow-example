use cosmwasm_std::{to_json_binary, Addr, Empty, StdError, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, MinterResponse};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    msg::{ConfigResponse, Cw20HookMsg, EscrowResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
};

#[test]
fn test_escrow_and_redeem() {
    let owner = Addr::unchecked("owner");
    let alice = Addr::unchecked("alice");
    let bob = Addr::unchecked("bob");

    let mut router: App = App::new(|_, _, _| {});

    // upload the contracts
    let escrow_id = router.store_code(contract_escrow());
    let usdc_id: u64 = router.store_code(contract_cw20());

    // instantiate the contracts
    let usdc_addr = router
        .instantiate_contract(
            usdc_id,
            owner.clone(),
            &cw20_base::msg::InstantiateMsg {
                name: "USDC".to_string(),
                symbol: "USDC".to_string(),
                decimals: 9, //see here
                initial_balances: vec![
                    Cw20Coin {
                        address: alice.to_string(),
                        amount: Uint128::from(1000u128),
                    },
                    Cw20Coin {
                        address: bob.to_string(),
                        amount: Uint128::from(1000u128),
                    },
                ],
                mint: Some(MinterResponse {
                    minter: owner.to_string(),
                    cap: None,
                }),
                marketing: None,
            },
            &[],
            "cw20",
            None,
        )
        .unwrap();

    let escrow_addr = router
        .instantiate_contract(
            escrow_id,
            owner.clone(),
            &InstantiateMsg {
                token: usdc_addr.to_string(),
            },
            &[],
            "engine",
            None,
        )
        .unwrap();

    // validate the config
    let msg = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(escrow_addr.clone(), &msg)
        .unwrap();
    assert_eq!(res.owner, owner);
    assert_eq!(res.token, usdc_addr.to_string());

    // now the cw20 and escrow contracts are deployed an initialized we can run actual some tests!

    // escrow funds into the contract
    let msg = Cw20ExecuteMsg::Send {
        // to the escrow contract
        contract: escrow_addr.to_string(),
        // send 100 USDC 
        amount: Uint128::from(100u128),
        // with this hook message
        msg: to_json_binary(&Cw20HookMsg::Escrow { time: 10 }).unwrap(),
    };

    let res = router
        .execute_contract(alice.clone(), usdc_addr.clone(), &msg, &[])
        .unwrap();
    // printing out to view in console for debug
    // println!("res.events[3]: {:?}", res.events[3]);
    assert_eq!("escrow", res.events[3].attributes[1].value);

    // duplicate escrow should fail (ContractError::ExistingEscrow)
    let err = router
        .execute_contract(alice.clone(), usdc_addr.clone(), &msg, &[])
        .unwrap_err();
    // printing out the error to view for debug
    // print!("err: {:?}", err);

    // check contract balance
    let msg = Cw20QueryMsg::Balance {
        address: escrow_addr.to_string(),
    };
    let res: BalanceResponse = router
        .wrap()
        .query_wasm_smart(usdc_addr.clone(), &msg)
        .unwrap();
    assert_eq!(res.balance, Uint128::from(100u128));

    let msg = QueryMsg::Escrow {
        address: alice.to_string(),
    };
    let res: EscrowResponse = router
        .wrap()
        .query_wasm_smart(escrow_addr.clone(), &msg)
        .unwrap();
    assert_eq!(res.amount, Uint128::from(100u128));
    assert_eq!(res.time, 1571797429u64);

    let msg_bob = QueryMsg::Escrow {
        address: bob.to_string(),
    };

    // check there is no escrow for Bob in the contract (will return a StdError)
    let err: Result<EscrowResponse, StdError> = router
        .wrap()
        .query_wasm_smart(escrow_addr.clone(), &msg_bob);

    print!("err: {:?}", err);

    // redeem funds from the escrow
    let msg = ExecuteMsg::Redeem {};

    // should fail as block has not moved
    router
        .execute_contract(alice.clone(), escrow_addr.clone(), &msg, &[])
        .unwrap_err();

    // move the block time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(20);
        block.height += 1;
    });

    let res = router
        .execute_contract(alice.clone(), escrow_addr.clone(), &msg, &[])
        .unwrap();
    assert_eq!("redeem", res.events[1].attributes[1].value);

    // check alice balance
    let msg = Cw20QueryMsg::Balance {
        address: alice.to_string(),
    };
    let res: BalanceResponse = router
        .wrap()
        .query_wasm_smart(usdc_addr.clone(), &msg)
        .unwrap();
    assert_eq!(res.balance, Uint128::from(1000u128));

    // check contract balance
    let msg = Cw20QueryMsg::Balance {
        address: escrow_addr.to_string(),
    };
    let res: BalanceResponse = router
        .wrap()
        .query_wasm_smart(usdc_addr.clone(), &msg)
        .unwrap();
    assert_eq!(res.balance, Uint128::zero());
}

fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn contract_escrow() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
    Box::new(contract)
}
