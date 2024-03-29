use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::hexdisplay::AsBytesRef;

use crate::generic_client::{
    load_project, DeployContract, Execution, ReadContract, WriteContract, API,
};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let code = std::fs::read("../contracts/flipper.wasm")?;
    let p = load_project("../contracts/flipper.contract")?;
    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", ["true".into()])?;

    let contract = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code,
    }
    .execute(&api)
    .await?;

    // get value
    let selector = transcoder.encode::<_, String>("get", [])?;

    let init_value = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| <bool>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert!(init_value);

    // flip flipper
    let selector = transcoder.encode::<_, String>("flip", [])?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        selector,
        value: 0,
    }
    .execute(&api)
    .await?;

    // get value
    let selector = transcoder.encode::<_, String>("get", [])?;

    let value = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| <bool>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert!(!value);

    Ok(())
}
