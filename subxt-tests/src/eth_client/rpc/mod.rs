//! various rpc caller
//!
//! manually call the node using subxt::API

use anyhow::Result;
use ethereum::TransactionV2;
use fc_rpc_core::types::{Index, RichBlock};
use parity_scale_codec::Decode;
use serde_json::json;
use sp_core::{serde::Serialize, Bytes, H160, H256, U256};
use subxt::{
    ext::scale_value::serde,
    rpc::{rpc_params, BlockNumber, ClientT, DeserializeOwned},
};

use crate::API;

#[cfg(test)]
pub mod endpoints;

pub struct EthErpcWrapper(pub API);

impl EthErpcWrapper {
    pub async fn get_balance(&self, target: H160, number: Option<BlockNumber>) -> Result<U256> {
        self.0
            .rpc()
            .client
            .request("eth_getBalance", rpc_params![target, number])
            .await
            .map_err(Into::into)
    }

    pub async fn get_transaction_counts(
        &self,
        target: H160,
        number: Option<BlockNumber>,
    ) -> Result<U256> {
        self.0
            .rpc()
            .client
            .request("eth_getTransactionCount", rpc_params![target, number])
            .await
            .map_err(Into::into)
    }

    pub async fn send_raw_transaction(&self, tx: Bytes) -> Result<H256> {
        self.0
            .rpc()
            .client
            .request("eth_sendRawTransaction", rpc_params![tx])
            .await
            .map_err(Into::into)
    }

    pub async fn get_chain_id(&self) -> Result<U256> {
        self.0
            .rpc()
            .client
            .request("eth_chainId", rpc_params!())
            .await
            .map_err(Into::into)
    }
    pub async fn get_block_number(&self) -> Result<U256> {
        self.0
            .rpc()
            .client
            .request("eth_blockNumber", rpc_params!())
            .await
            .map_err(Into::into)
    }

    pub async fn get_block_by_number(
        &self,
        number: BlockNumber,
        full: bool,
    ) -> Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request("eth_getBlockByNumber", rpc_params![number, full])
            .await
            .map_err(Into::into)
    }

    pub async fn get_block_by_hash(
        &self,
        hash: H256,
        full: bool,
    ) -> Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request("eth_getBlockByHash", rpc_params![hash, full])
            .await
            .map_err(Into::into)
    }
    // pub async fn get_uncle_by_block_and_index(&self, )
    // pub async fn get_uncle_by_block_number_and_index()
    // pub async fn get_uncle_count_by_block_number()
    pub async fn get_block_transaction_count_by_hash(
        &self,
        hash: H256,
    ) -> anyhow::Result<Option<U256>> {
        self.0
            .rpc()
            .client
            .request("eth_getBlockTransactionCountByHash", rpc_params![hash])
            .await
            .map_err(Into::into)
    }
    pub async fn get_block_transaction_count_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<Option<U256>> {
        self.0
            .rpc()
            .client
            .request("eth_getBlockTransactionCountByNumber", rpc_params![number])
            .await
            .map_err(Into::into)
    }
    // pub async fn get_accounts()
    // pub async fn get_code()
    // pub async fn get_gas_price()
    pub async fn get_transaction_receipt(
        &self,
        hash: H256,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request("eth_getTransactionReceipt", rpc_params![hash])
            .await
            .map_err(Into::into)
    }

    pub async fn get_transaction_by_hash(
        &self,
        hash: H256,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request("eth_getTransactionByHash", rpc_params![hash])
            .await
            .map_err(Into::into)
    }

    pub async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: H256,
        index: usize,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request(
                "eth_getTransactionByBlockHashAndIndex",
                rpc_params![hash, index],
            )
            .await
            .map_err(Into::into)
    }

    pub async fn get_transaction_by_block_number_and_index(
        &self,
        number: BlockNumber,
        index: usize,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        self.0
            .rpc()
            .client
            .request(
                "eth_getTransactionByBlockNumberAndIndex",
                rpc_params![number, index],
            )
            .await
            .map_err(Into::into)
    }
}

#[tokio::test]
async fn check_endpoints() -> anyhow::Result<()> {
    let api = API::from_url("wss://laguna-chain-dev.hydrogenx.live:443".to_string()).await?;

    let caller = EthErpcWrapper(api);

    let balance = caller.get_balance(Default::default(), None).await.unwrap();
    assert_eq!(balance, 100_u32.into());

    let nonce = caller
        .get_transaction_counts(Default::default(), None)
        .await
        .unwrap();
    assert_eq!(nonce, 0_u32.into());

    Ok(())
}
