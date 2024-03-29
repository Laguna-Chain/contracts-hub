use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::hexdisplay::AsBytesRef;

use crate::generic_client::{
    load_project, node, DeployContract, Execution, ReadContract, API, GAS_LIMIT,
};

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;
    let code = std::fs::read("../contracts/builtins2.wasm")?;

    let p = load_project("../contracts/builtins2.contract")?;
    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", [])?;
    let deployed = DeployContract {
        caller: sp_keyring::AccountKeyring::Alice,
        selector,
        value: 0,
        code,
    }
    .execute(&api)
    .await?;

    // check blake2_128
    let input_str = "Call me Ishmael.";

    let selector = transcoder.encode(
        "hash_blake2_128",
        [format!("0x{}", hex::encode(&input_str))],
    )?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: deployed.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await?;

    let expected = hex::decode("56691483d63cac66c38c168c703c6f13")?;
    assert_eq!(rv.return_value, expected);

    // check blake2_256
    let selector = transcoder.encode(
        "hash_blake2_256",
        [format!("0x{}", hex::encode(&input_str))],
    )?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: deployed.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await?;

    let expected = hex::decode("1abd7330c92d835b5084219aedba821c3a599d039d5b66fb5a22ee8e813951a8")?;
    assert_eq!(rv.return_value, expected);

    // check block_height
    let selector = transcoder.encode::<_, String>("block_height", [])?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: deployed.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await?;

    let decoded = u64::decode(&mut rv.return_value.as_bytes_ref())? as i64;

    let key = node::storage().system().number();

    let rpc_block_number = api.storage().fetch_or_default(&key, None).await?;

    assert!((decoded - rpc_block_number as i64).abs() <= 3);

    // check gas burn
    let selector = transcoder.encode::<_, String>("burn_gas", [format!("{}", 0_u64)])?;

    let rv = ReadContract {
        caller: sp_keyring::AccountKeyring::Alice,
        contract_address: deployed.contract_address.clone(),
        value: 0,
        selector,
    }
    .execute(&api)
    .await?;

    let gas_left = u64::decode(&mut rv.return_value.as_bytes_ref())?;

    assert!(GAS_LIMIT > gas_left);

    let mut previous_used = GAS_LIMIT - gas_left;

    for i in 1_u64..100 {
        // check gas burn
        let selector = transcoder.encode::<_, String>("burn_gas", [format!("{}", i)])?;

        let rv = ReadContract {
            caller: sp_keyring::AccountKeyring::Alice,
            contract_address: deployed.contract_address.clone(),
            value: 0,
            selector,
        }
        .execute(&api)
        .await?;

        let gas_left = u64::decode(&mut rv.return_value.as_bytes_ref())?;

        assert!(GAS_LIMIT > gas_left);

        let gas_used = GAS_LIMIT - gas_left;

        assert!(gas_used > previous_used);
        assert!(gas_used - previous_used < 10_u64.pow(6));
        assert!(gas_used - previous_used > 10_u64.pow(4));

        previous_used = gas_used;
    }

    Ok(())
}
