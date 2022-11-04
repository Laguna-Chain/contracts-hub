use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::{ecdsa, hexdisplay::AsBytesRef, Bytes, Pair, U256};
use subxt::tx::PairSigner;

use crate::eth_client::rpc::EthErpcWrapper;
use crate::eth_client::{PayloadFactory, SignPayload, TxWrapper};
use crate::generic_client::{load_project, DeployContract, Execution, ReadContract, WriteContract};
use crate::utils::to_eth_address;
use crate::{node, API};
use ethereum::{EIP1559TransactionMessage, EIP2930TransactionMessage, LegacyTransactionMessage};
use rlp::{Decodable, Encodable};
use sp_keyring::AccountKeyring;

#[tokio::test]
async fn case() -> anyhow::Result<()> {
    let alice = PairSigner::new(AccountKeyring::Alice.pair());

    let api = API::from_url(
        std::env::var("ENDPOINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;

    let eth_client_wrapper = EthErpcWrapper(api.clone());

    let code = std::fs::read("../contracts/flipper.wasm")?;
    let p = load_project("../contracts/flipper.contract")?;
    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", ["true".into()])?;

    let key = "//Alice";
    let pair = ecdsa::Pair::from_string(key, None).unwrap();
    let eth_alice = to_eth_address(pair.public())?;

    let prefund = node::tx().evm_compat().transfer(
        Decode::decode(&mut &eth_alice.encode()[..])?,
        10_u128.pow(18),
    );

    let balance = eth_client_wrapper.get_balance(eth_alice, None).await?;

    if balance <= U256::from(10_u32).pow(18_u32.into()) {
        api.tx()
            .sign_and_submit_then_watch_default(&prefund, &alice)
            .await?
            .wait_for_in_block()
            .await?;
    }

    let nonce = eth_client_wrapper
        .get_transaction_counts(eth_alice, None)
        .await?;

    let mut create_payload = TxWrapper::<LegacyTransactionMessage>::create(
        0_u32.into(),
        code.into(),
        selector.into(),
        Bytes::from(vec![nonce.as_u128() as u8]),
    );

    create_payload.chain_id.replace(1000);
    create_payload.nonce = nonce;
    create_payload.gas_limit = U256::from(10_u32).pow(18_u32.into());
    create_payload.gas_price = U256::from(1_u32);

    let raw =
        TxWrapper::<LegacyTransactionMessage>::sign(create_payload, &sp_core::H256(pair.seed()))
            .rlp_bytes()
            .to_vec();

    let tx_id = eth_client_wrapper.send_raw_transaction(raw.into()).await?;

    dbg!(tx_id);

    Ok(())
}
