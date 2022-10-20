use anyhow::Result;
use k256::{elliptic_curve::sec1::ToEncodedPoint, PublicKey};
use sp_core::keccak_256;
use sp_core::{crypto::AccountId32, ecdsa::Public, ByteArray, H160};

use crate::{
    node::{self, runtime_types::primitives::currency::CurrencyId},
    API,
};

pub async fn free_balance_of(api: &API, addr: AccountId32) -> anyhow::Result<u128> {
    let key = node::storage().tokens().accounts(
        addr,
        CurrencyId::NativeToken(node::runtime_types::primitives::currency::TokenId::Laguna),
    );
    let val = api.storage().fetch_or_default(&key, None).await?;

    Ok(val.free)
}

pub fn to_eth_address(public: Public) -> Result<H160> {
    let pk = PublicKey::from_sec1_bytes(public.as_slice())?;

    let uncompressed = pk.to_encoded_point(false);
    // convert to ETH address
    <[u8; 20]>::try_from(keccak_256(&uncompressed.as_bytes()[1..])[12..].as_ref())
        .map(H160)
        .map_err(Into::into)
}
