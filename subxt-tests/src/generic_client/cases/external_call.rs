use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::{crypto::AccountId32, hexdisplay::AsBytesRef};

use crate::generic_client::{
    load_project, DeployContract, Execution, ReadContract, WriteContract, API,
};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let caller_code = std::fs::read("../contracts/caller.wasm")?;
    let callee_code = std::fs::read("../contracts/callee.wasm")?;
    let callee2_code = std::fs::read("../contracts/callee2.wasm")?;

    let p_caller = load_project("../contracts/caller.contract")?;
    let t_caller = ContractMessageTranscoder::new(&p_caller);

    let p_callee = load_project("../contracts/callee.contract")?;
    let t_callee = ContractMessageTranscoder::new(&p_callee);

    let p_callee2 = load_project("../contracts/callee2.contract")?;
    let t_callee2 = ContractMessageTranscoder::new(&p_callee2);

    let selector = t_caller.encode::<_, String>("new", [])?;

    let caller = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector: selector.clone(),
        value: 0,
        code: caller_code,
    }
    .execute(&api)
    .await?;

    let callee = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector: selector.clone(),
        value: 0,
        code: callee_code,
    }
    .execute(&api)
    .await?;

    let callee2 = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector: selector.clone(),
        value: 0,
        code: callee2_code,
    }
    .execute(&api)
    .await?;

    // setX on callee
    let selector = t_callee.encode::<_, String>("set_x", [format!("102")])?;

    // let selector = build_selector(
    //     "250c2025",
    //     Some(&mut |s| {
    //         102_i64.encode_to(s);
    //     }),
    // )?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: callee.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    // getX on callee
    let selector = t_callee.encode::<_, String>("get_x", [])?;

    let res1 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: callee.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <i64>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(res1, 102);

    // whoAmI on caller
    let selector = t_caller.encode::<_, String>("who_am_i", [])?;

    let res2 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: caller.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <AccountId32>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(res2, caller.contract_address.clone());

    // doCall on caller
    let selector = t_caller.encode::<_, String>(
        "do_call",
        [
            format!("0x{}", hex::encode(callee.contract_address.clone())),
            "13123".to_string(),
        ],
    )?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: caller.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    // getX on callee
    let selector = t_callee.encode::<_, String>("get_x", [])?;

    let res3 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: callee.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <i64>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(res3, 13123);

    // doCall2 on caller
    let selector = t_caller.encode::<_, String>(
        "do_call2",
        [
            format!("0x{}", hex::encode(callee.contract_address.clone())),
            "20000".to_string(),
        ],
    )?;

    let res4 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: caller.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| <i64>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_eq!(res4, 33123);

    // doCall3 on caller
    let selector = t_caller.encode::<_, String>(
        "do_call3",
        [
            format!("0x{}", hex::encode(callee.contract_address.clone())),
            format!("0x{}", hex::encode(callee2.contract_address.clone())),
            format!("{:?}", [3_i64, 5, 7, 9]),
            "\"yo\"".to_string(),
        ],
    )?;

    let res5 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: caller.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| {
        <(i64, String)>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into)
    })?;

    assert_eq!(res5, (24, "my name is callee".to_string()));

    // doCall4 on caller
    let selector = t_caller.encode::<_, String>(
        "do_call4",
        [
            format!("0x{}", hex::encode(callee.contract_address.clone())),
            format!("0x{}", hex::encode(callee2.contract_address.clone())),
            format!("{:?}", [1_i64, 2, 3, 4]),
            "\"asda\"".to_string(),
        ],
    )?;

    let res6 = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: caller.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await
    .and_then(|v| {
        <(i64, String)>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into)
    })?;

    assert_eq!(res6, (10, "x:asda".to_string()));

    Ok(())
}
