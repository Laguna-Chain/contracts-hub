use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::{hexdisplay::AsBytesRef, keccak_256};

use crate::generic_client::{
    load_project, DeployContract, Execution, ReadContract, WriteContract, API,
};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let code = std::fs::read("../contracts/randomizer.wasm")?;

    let p = load_project("../contracts/randomizer.contract")?;

    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", [])?;

    let contract = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code,
    }
    .execute(&api)
    .await?;

    let selector =
        transcoder.encode::<_, _>("get_random", [format!("{:?}", "01234567".as_bytes())])?;

    let rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        value: 0,
        selector: selector.clone(),
    }
    .execute(&api)
    .await
    .and_then(|v| <[u8; 32]>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    WriteContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await?;

    let selector = transcoder.encode::<_, String>("value", [])?;

    let tx_rs = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: contract.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await
    .and_then(|v| <[u8; 32]>::decode(&mut v.return_value.as_bytes_ref()).map_err(Into::into))?;

    assert_ne!(rs, [0_u8; 32]);
    assert_ne!(tx_rs, [0_u8; 32]);
    assert_ne!(rs, tx_rs);

    Ok(())
}
