use crate::generic_client::Contract;
use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::Decode;
use sp_core::hexdisplay::AsBytesRef;

#[tokio::test]
async fn access_env_utils_from_solidity() -> anyhow::Result<()> {
    const ALICE: sp_keyring::AccountKeyring = sp_keyring::AccountKeyring::Alice;

    let api = crate::API::from_url(
        std::env::var("END_POINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    // 1A. Deploy the system-contract (env_utils)
    let mut system_contract = Contract::new("../contracts/env_utils.contract")?;
    system_contract
        .deploy_as_system_contract(&api, None, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("new", []).unwrap()
        })
        .await?;

    let system_contract_addr = system_contract.address.unwrap();

    // 1B. Deploy the sample solidity contract
    let mut contract = Contract::new("../contracts/TestEnvUtils.contract")?;
    contract
        .deploy(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("new", [format!("0x{}", hex::encode(&system_contract_addr))])
                .unwrap()
        })
        .await?;

    let contract_addr = contract.address.as_ref().unwrap();

    // 2. Test API -> is_contract

    // A. Passing non-contract address (ALICE); Should return false
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>(
                "is_contract",
                [format!("0x{}", hex::encode(&ALICE.to_account_id()))],
            )
            .unwrap()
        })
        .await?;
    assert!(<bool>::decode(&mut rv.as_bytes_ref())? == false);

    // B. Passing contract address (SELF); Should return true
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>(
                "is_contract",
                [format!("0x{}", hex::encode(&contract_addr))],
            )
            .unwrap()
        })
        .await?;
    assert!(<bool>::decode(&mut rv.as_bytes_ref())?);

    // 3. Test API -> code_hash

    // A. Passing non-contract address should return (false, bytes32(0))
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>(
                "code_hash",
                [format!("0x{}", hex::encode(&ALICE.to_account_id()))],
            )
            .unwrap()
        })
        .await?;

    let res = <(bool, [u8; 32])>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(res, (false, [0u8; 32]));

    // B. Passing contract address should return (true, code_hash)
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("code_hash", [format!("0x{}", hex::encode(&contract_addr))])
                .unwrap()
        })
        .await?;

    let res = <(bool, [u8; 32])>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(res, (true, contract.code_hash.0));

    // 4. Test API -> own_code_hash
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("own_code_hash", []).unwrap()
        })
        .await?;

    let res = <[u8; 32]>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(res, contract.code_hash.0);

    // 5. Test API -> ecdsa_to_eth_address
    let pubkey: [u8; 33] = [
        3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127, 31, 13,
        58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ];

    let expected_eth_addr = [
        253, 240, 181, 194, 143, 66, 163, 109, 18, 211, 78, 49, 177, 94, 159, 79, 207, 37, 21, 191,
    ];

    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("ecdsa_to_eth_address", [format!("{:?}", pubkey)])
                .unwrap()
        })
        .await?;

    let res = <(bool, [u8; 20])>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(res, (true, expected_eth_addr));

    // 6. Test API -> ecdsa_recover
    let signature: [u8; 65] = [
        195, 218, 227, 165, 226, 17, 25, 160, 37, 92, 142, 238, 4, 41, 244, 211, 18, 94, 131, 116,
        231, 116, 255, 164, 252, 248, 85, 233, 173, 225, 26, 185, 119, 235, 137, 35, 204, 251, 134,
        131, 186, 215, 76, 112, 17, 192, 114, 243, 102, 166, 176, 140, 180, 124, 213, 102, 117,
        212, 89, 89, 92, 209, 116, 17, 28,
    ];

    let msg_hash: [u8; 32] = [
        167, 124, 116, 195, 220, 156, 244, 20, 243, 69, 1, 98, 189, 205, 79, 108, 213, 78, 65, 65,
        230, 30, 17, 37, 184, 220, 237, 135, 1, 209, 101, 229,
    ];

    let expected_compressed_pubkey: [u8; 33] = [
        3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127, 31, 13,
        58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ];

    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>(
                "ecdsa_recover",
                [format!("{:?}", signature), format!("{:?}", msg_hash)],
            )
            .unwrap()
        })
        .await?;

    let res = <(bool, [u8; 33])>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(res, (true, expected_compressed_pubkey));

    Ok(())
}
