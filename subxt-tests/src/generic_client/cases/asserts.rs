use crate::generic_client::{node, Contract, WriteContract};
use contract_transcode::ContractMessageTranscoder;
use hex::FromHex;
use parity_scale_codec::{Decode, Encode};
use sp_core::hexdisplay::AsBytesRef;
use subxt::metadata::ErrorMetadata;

use crate::generic_client::API;

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let mut contract = Contract::new("../contracts/asserts.contract")?;

    contract
        .deploy(
            &api,
            sp_keyring::AccountKeyring::Alice,
            0,
            &|t: ContractMessageTranscoder<'_>| t.encode::<_, String>("new", []).unwrap(),
        )
        .await?;

    let rv = contract
        .try_call(
            &api,
            sp_keyring::AccountKeyring::Alice,
            0,
            &|t: ContractMessageTranscoder<'_>| t.encode::<_, String>("var", []).unwrap(),
        )
        .await?;

    let output = i64::decode(&mut rv.as_bytes_ref())?;
    assert!(output == 1);

    // read should fail
    let res = contract
        .try_call(
            &api,
            sp_keyring::AccountKeyring::Alice,
            0,
            &|t: ContractMessageTranscoder<'_>| {
                t.encode::<_, String>("test_assert_rpc", []).unwrap()
            },
        )
        .await;

    if let Err(r) = res {
        assert!(r.to_string().contains("ContractTrapped"));
    }

    // write should failed
    let res = contract
        .call(
            &api,
            sp_keyring::AccountKeyring::Alice,
            0,
            &|t: ContractMessageTranscoder<'_>| {
                t.encode::<_, String>("test_assert_rpc", []).unwrap()
            },
        )
        .await;

    if let Err(r) = res {
        assert!(r.to_string().contains("ContractTrapped"));
    }

    // state should not change after failed operation
    let rv = contract
        .try_call(
            &api,
            sp_keyring::AccountKeyring::Alice,
            0,
            &|t: ContractMessageTranscoder<'_>| t.encode::<_, String>("var", []).unwrap(),
        )
        .await?;

    let output = i64::decode(&mut rv.as_bytes_ref())?;
    assert!(output == 1);

    Ok(())
}
