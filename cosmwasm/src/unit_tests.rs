use crate::contract::{instantiate, query};
use crate::msg::{ConfigResponse, InstantiateMsg, QueryMsg};

use cosmwasm_std::{from_json};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

#[test]
fn test_instantiation() {
    let mut deps = mock_dependencies();
    let info = mock_info("owner", &[]);
    let msg = InstantiateMsg {
        token: "token".to_string(),
    };
    // call the actual instantiate function on the contract under test
    instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_json(&res).unwrap();
    // let info = mock_info("owner", &[]);
    assert_eq!(
        config,
        ConfigResponse {
            owner: info.sender.to_string(),
            token: "token".to_string(),
        }
    );
}

#[test]
fn test_query_escrow_error() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        token: "token".to_string(),
    };
    let info = mock_info("owner", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Escrow {
            address: "user".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(res.to_string(), "Generic error: No escrow found");
}
