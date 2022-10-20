//! various rpc caller
//!
//! manually call the node using subxt::API

use anyhow::Result;
use ethereum::TransactionV2;
use parity_scale_codec::Decode;
use serde_json::json;
use sp_core::{serde::Serialize, Bytes, H160, H256, U256};
use subxt::rpc::{rpc_params, BlockNumber, ClientT, DeserializeOwned};

use crate::API;

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
}

#[tokio::test]
async fn check_endpoints() {
    let api = API::new().await.unwrap();
    let caller = EthErpcWrapper(api);

    let balance = caller.get_balance(Default::default(), None).await.unwrap();
    assert_eq!(balance, 0_u32.into());

    let nonce = caller
        .get_transaction_counts(Default::default(), None)
        .await
        .unwrap();
    assert_eq!(nonce, 0_u32.into());
}
