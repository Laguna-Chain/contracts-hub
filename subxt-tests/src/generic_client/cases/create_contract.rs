use std::str::FromStr;

use contract_transcode::ContractMessageTranscoder;
use hex::FromHex;
use parity_scale_codec::{Decode, Encode};
use sp_core::{crypto::AccountId32, hexdisplay::AsBytesRef, H256};

use crate::utils::free_balance_of;

use crate::generic_client::{
    load_project, node, DeployContract, Execution, ReadContract, ReadLayout, WriteContract, API,
};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let creator_code = std::fs::read("../contracts/creator.wasm")?;

    let p_creator = load_project("../contracts/creator.contract")?;
    let t_creator = ContractMessageTranscoder::new(&p_creator);

    let child_code = std::fs::read("../contracts/child.wasm")?;

    let p_child = load_project("../contracts/child.contract")?;
    let t_child = ContractMessageTranscoder::new(&p_child);

    let selector = t_creator.encode::<_, String>("new", [])?;

    let creator = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 10_u128.pow(16),
        code: creator_code,
    }
    .execute(&api)
    .await?;

    let selector = t_child.encode::<_, String>("new", [])?;

    // upload child code hash for creator to use it
    DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code: child_code,
    }
    .execute(&api)
    .await?;

    let selector = t_creator.encode::<_, String>("create_child", [])?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: creator.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    let selector = t_creator.encode::<_, String>("call_child", [])?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: creator.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| String::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(rv, "child");

    let selector = t_creator.encode::<_, String>("call_child", [])?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: creator.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| String::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(rv, "child");

    let selector = t_creator.encode::<_, String>("c", [])?;

    let child_addr = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: creator.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| <AccountId32>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    let child_balance_rpc = free_balance_of(&api, child_addr).await?;

    assert!(10_u128.pow(15) - child_balance_rpc < 10_u128.pow(11));

    Ok(())
}
