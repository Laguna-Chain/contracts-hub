use crate::{
    eth_client::{PayloadFactory, SignPayload, TxWrapper},
    generic_client::load_project,
    node,
    utils::to_eth_address,
};

use super::*;
use contract_transcode::ContractMessageTranscoder;
use ethereum::LegacyTransactionMessage;
use parity_scale_codec::Encode;
use rlp::Encodable;
use sp_core::Pair;
use sp_keyring::AccountKeyring;
use std::str::FromStr;
use subxt::{
    config::WithExtrinsicParams,
    tx::{PairSigner, PolkadotExtrinsicParams, Signer},
    SubstrateConfig,
};

#[tokio::test]
async fn get_balance() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api.clone());
    let target: H160 = H160::from_slice("0xl33700000000000000".as_bytes());
    // let alice = PairSigner::new(AccountKeyring::Alice.pair());
    let alice = PairSigner::new(AccountKeyring::Alice.pair());
    let init_balance = caller.get_balance(target, None).await.unwrap();
    // transfer some tokens from Alice to the Default address
    let fund_transfer = node::tx()
        .evm_compat()
        .transfer(Decode::decode(&mut &target.encode()[..])?, 10);

    api.tx()
        .sign_and_submit_then_watch_default(&fund_transfer, &alice)
        .await?
        .wait_for_in_block()
        .await?;

    let curr_balance = caller.get_balance(target, None).await.unwrap();

    assert_eq!(curr_balance, init_balance + U256::from(10));

    Ok(())
}

#[tokio::test]
pub async fn get_chain_id() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_accounts() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_block_by_hash() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);

    let latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    // get the latest block
    let latest_block_by_number = caller
        .get_block_by_number(latest_block_number.into(), false)
        .await
        .expect("Fetching latest block failed")
        .unwrap();
    // get the block hash
    let latest_block_hash_in_bytes =
        &*Bytes::from_str(latest_block_by_number["hash"].as_str().unwrap()).unwrap();

    let latest_block_hash = H256::from_slice(latest_block_hash_in_bytes);

    // Now fetch the same block using the block hash
    let latest_block_by_hash = caller
        .get_block_by_hash(latest_block_hash, false)
        .await
        .expect("Fetching latest block by hash failed")
        .unwrap();

    // extract the block hash and compare it with the one fetched using the get_block_by_number() earlier
    let block_hash_by_hash = latest_block_by_hash["hash"].as_str().unwrap();
    let block_hash_by_number = latest_block_by_number["hash"].as_str().unwrap();

    assert_eq!(block_hash_by_hash, block_hash_by_number);

    Ok(())
}

#[tokio::test]
pub async fn get_block_by_number() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);

    let latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    // get the latest block
    let latest_block = caller
        .get_block_by_number(latest_block_number.into(), true)
        .await
        .expect("Failed fetching latest block")
        .unwrap();

    let block_number_from_retrieved_block_in_hex = latest_block["number"].as_str().unwrap();
    // Decode the hex representation of the block number to decimals
    let block_number_from_retrieved_block = u64::from_str_radix(
        block_number_from_retrieved_block_in_hex.get(2..).unwrap(),
        16,
    )
    .unwrap();

    assert_eq!(latest_block_number, block_number_from_retrieved_block);

    // fetch a block from the past
    let older_block_number = latest_block_number.checked_div(2).unwrap();

    let older_block = caller
        .get_block_by_number(older_block_number.into(), false)
        .await
        .expect("Falied fetching the older block")
        .unwrap();

    let older_block_number_from_retrieved_block_in_hex = older_block["number"].as_str().unwrap();
    let older_block_number_from_retrieved_block = u64::from_str_radix(
        older_block_number_from_retrieved_block_in_hex
            .get(2..)
            .unwrap(),
        16,
    )
    .unwrap();

    assert_eq!(older_block_number, older_block_number_from_retrieved_block);

    Ok(())
}

#[tokio::test]
pub async fn get_block_transaction_count_by_hash() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);

    let block_number = caller.get_block_number().await.unwrap().as_u64();

    let block = caller
        .get_block_by_number(block_number.into(), true)
        .await
        .expect("Failed fetching block")
        .unwrap();

    // get the count of the number of transactions in the block
    let count = U256::from(block["transactions"].as_array().unwrap().len() as u128);
    // extract the block hash
    let block_hash = H256::from_slice(&*Bytes::from_str(block["hash"].as_str().unwrap()).unwrap());

    println!("-->>> {}", block);
    // get the block transaction count using the rpc
    let block_tx_count = caller
        .get_block_transaction_count_by_hash(block_hash)
        .await
        .expect("Failed fetching transaction count")
        .unwrap();

    assert_eq!(count, block_tx_count);

    Ok(())
}

#[tokio::test]
pub async fn get_block_transaction_count_by_number() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);

    // get the block number 500 blocks away from the latest one
    let block_number = caller.get_block_number().await.unwrap().as_u64() - 500;

    let block = caller
        .get_block_by_number(block_number.into(), true)
        .await
        .expect("Fetching block failed")
        .unwrap();
    // count the number of transactions included in the block
    let count = U256::from(block["transactions"].as_array().unwrap().len() as u128);
    // get the transaction count returned from the rpc server
    let tx_count = caller
        .get_block_transaction_count_by_number(block_number.into())
        .await
        .expect("Failed fetching tx count")
        .unwrap();

    assert_eq!(count, tx_count);
    Ok(())
}

#[tokio::test]
pub async fn get_transaction_by_hash() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api.clone());

    // Idea: make an evm extrinsic call and wait for it to be included in the block.
    // Then fetch that block and extract the transaction, try to get the same transaction
    // using its hash and compare it with the original.
    // Prepare to deploy a smart contract
    let raw = contract_create_helper(&caller).await.unwrap();

    let tx_id = caller.send_raw_transaction(raw.into()).await?;

    // wait for at least 6 seconds for the transaction to be included in the block
    std::thread::sleep(std::time::Duration::from_secs(9));
    // Get the latest block number
    let mut latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    let mut tx_hash = "".to_string();
    let mut flag = true;
    // iterate backwards till you find a block with an evm transaction
    while flag {
        let block = caller
            .get_block_by_number(latest_block_number.into(), true)
            .await
            .expect("Failed fetching block")
            .unwrap();
        // iterate till you get the required tx
        for _tx in block["transactions"].as_array().unwrap().into_iter() {
            if H256::from_slice(&*Bytes::from_str(_tx["hash"].as_str().unwrap()).unwrap()) == tx_id
            {
                tx_hash = _tx["hash"].as_str().unwrap().to_string();
                flag = false;
                break;
            }
        }
        latest_block_number -= 1;
    }

    // fetch the transaction by quering the rpc server
    let tx_retrieved = caller
        .get_transaction_by_hash(tx_id)
        .await
        .expect("Transaction fetching failed")
        .unwrap();

    assert_eq!(tx_retrieved["hash"].as_str().unwrap(), tx_hash);

    Ok(())
}

#[tokio::test]
pub async fn get_transaction_by_block_hash_and_index() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);
    // send a transaction to be included in a block, then fetch that block and extract the transaction.
    // Fetch the same transaction by quering the rpc server and assert that both the transactions are same.

    // get the signed ethereum transaction payload
    let eth_payload = contract_create_helper(&caller).await.unwrap();
    caller
        .send_raw_transaction(eth_payload.into())
        .await
        .unwrap();

    // wait for at least 6 seconds for the tx to be included in a block
    std::thread::sleep(std::time::Duration::from_secs(9));

    // get the latest block number and start iterating the blocks backwards until an eth transaction is found.
    let mut latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    let mut tx_hash = "".to_string();
    let mut block_hash: H256 = Default::default();

    loop {
        let block = caller
            .get_block_by_number(latest_block_number.into(), true)
            .await
            .expect("Failed fetching block")
            .unwrap();

        if block["transactions"].as_array().unwrap().len() > 0 {
            tx_hash = block["transactions"].as_array().unwrap()[0]["hash"]
                .as_str()
                .unwrap()
                .to_string();
            block_hash =
                H256::from_slice(&*Bytes::from_str(block["hash"].as_str().unwrap()).unwrap());
            break;
        }

        latest_block_number -= 1;
    }

    // fetch the transaction using the rpc method
    let tx_rpc = caller
        .get_transaction_by_block_hash_and_index(block_hash, 0usize)
        .await
        .expect("Failed fetching transaction")
        .unwrap();
    // Extract the tx hash
    let tx_rpc_hash = tx_rpc["hash"].as_str().unwrap().to_string();

    assert_eq!(tx_hash, tx_rpc_hash);
    Ok(())
}

#[tokio::test]
pub async fn get_transaction_by_block_number_and_index() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api.clone());

    // send a transaction to be included in a block, then fetch that block and extract the transaction.
    // Fetch the same transaction by quering the rpc server and assert that both the transactions are same.

    // get the signed ethereum transaction payload
    let eth_payload = contract_create_helper(&caller).await.unwrap();
    caller
        .send_raw_transaction(eth_payload.into())
        .await
        .unwrap();

    // wait for at least 6 seconds for the tx to be included in a block
    std::thread::sleep(std::time::Duration::from_secs(9));

    // get the latest block number and start iterating the blocks backwards until an eth transaction is found.
    let mut latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    let mut tx_hash = "".to_string();

    loop {
        let block = caller
            .get_block_by_number(latest_block_number.into(), true)
            .await
            .expect("Failed fetching block")
            .unwrap();

        if block["transactions"].as_array().unwrap().len() > 0 {
            tx_hash = block["transactions"].as_array().unwrap()[0]["hash"]
                .as_str()
                .unwrap()
                .to_string();
            break;
        }

        latest_block_number -= 1;
    }

    // fetch the transaction using the rpc method
    let tx_rpc = caller
        .get_transaction_by_block_number_and_index(latest_block_number.into(), 0usize)
        .await
        .expect("Failed fetching transaction")
        .unwrap();
    // Extract the transaction hash
    let tx_rpc_hash = tx_rpc["hash"].as_str().unwrap().to_string();

    assert_eq!(tx_hash, tx_rpc_hash);

    Ok(())
}

#[tokio::test]
pub async fn get_transaction_receipt() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api.clone());

    // IDEA: submit an eth transaction and immediatly get its receipt, which should get fetched directly from the
    // tx pool and assert the block number to be None. Next wait for the tx to be included in a block, then fetch it
    // and assert that its block number is equal to the block it was included.

    // get the signed ethereum transaction payload
    let eth_payload = contract_create_helper(&caller).await.unwrap();
    let tx_hash = caller
        .send_raw_transaction(eth_payload.into())
        .await
        .unwrap();
    let tx_hash_str = String::from_utf8_lossy(tx_hash.as_bytes()).to_string();

    // Immediatly fetch the transaction's receipt while it is still in the tx pool.
    let mut tx_receipt = caller
        .get_transaction_receipt(tx_hash)
        .await
        .expect("Failed fetching tx receipt")
        .unwrap();

    // The block number should be null as the tx is still not yet included in a block
    assert!(tx_receipt["blockHash"].is_null());
    // println!("--->>> {}", tx_receipt["blockHash"]);

    // wait for at least 6 seconds for the tx to be included in a block
    std::thread::sleep(std::time::Duration::from_secs(9));

    // get the latest block number and start iterating the blocks backwards until an eth transaction is found.
    // Get the latest block number
    let mut latest_block_number = caller.get_block_number().await.unwrap().as_u64();
    let mut block_hash = "".to_string();
    let mut flag = true;
    // iterate backwards till you find a block with an evm transaction
    while flag {
        let block = caller
            .get_block_by_number(latest_block_number.into(), true)
            .await
            .expect("Failed fetching block")
            .unwrap();
        // iterate till you get the required tx
        for _tx in block["transactions"].as_array().unwrap().into_iter() {
            if H256::from_slice(&*Bytes::from_str(_tx["hash"].as_str().unwrap()).unwrap())
                == tx_hash
            {
                block_hash = block["hash"].as_str().unwrap().to_string();
                flag = false;
                break;
            }
        }
        latest_block_number -= 1;
    }

    // fetch the tx receipt again knowing that the tx would be included in a block
    tx_receipt = caller
        .get_transaction_receipt(tx_hash)
        .await
        .expect("Failed fetching tx receipt")
        .unwrap();

    // The block hash included in the tx receipt must be the same as the one obtained previously by
    // searching the tx by iterating over the blocks
    assert_eq!(block_hash, tx_receipt["blockHash"]);
    Ok(())
}

#[tokio::test]
pub async fn get_code() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_eth_subscribe() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_eth_unsubscribe() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_storage_at() -> anyhow::Result<()> {
    Ok(())
}

#[tokio::test]
pub async fn get_() -> anyhow::Result<()> {
    Ok(())
}

pub async fn contract_create_helper(caller: &EthErpcWrapper) -> anyhow::Result<Vec<u8>> {
    let code = std::fs::read("../contracts/flipper.wasm")?;
    let p = load_project("../contracts/flipper.contract")?;
    let transcoder = ContractMessageTranscoder::new(&p);

    let selector = transcoder.encode::<_, String>("new", ["true".into()])?;

    let mut create_payload = TxWrapper::<LegacyTransactionMessage>::create(
        0_u32.into(),
        code.into(),
        selector.into(),
        Bytes::from(vec![]),
    );

    let key = "//Alice";
    let pair = sp_core::ecdsa::Pair::from_string(key, None).unwrap();
    let eth_alice = to_eth_address(pair.public())?;
    let nonce = caller.get_transaction_counts(eth_alice, None).await?;

    create_payload.chain_id.replace(1000);
    create_payload.nonce = nonce;
    create_payload.gas_limit = U256::from(10_u32).pow(18_u32.into());
    create_payload.gas_price = U256::from(1_u32);

    let raw = TxWrapper::<LegacyTransactionMessage>::sign(create_payload, &H256(pair.seed()))
        .rlp_bytes()
        .to_vec();

    Ok(raw)
}
