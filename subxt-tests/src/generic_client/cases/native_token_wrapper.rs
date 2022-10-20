use crate::generic_client::{
    node::{
        self,
        runtime_types::primitives::currency::{CurrencyId, TokenId},
    },
    Contract,
};
use crate::utils::free_balance_of;

use contract_transcode::ContractMessageTranscoder;
use parity_scale_codec::{Decode, Encode};
use sp_core::{hexdisplay::AsBytesRef, U256};
use sp_keyring::AccountKeyring;

#[tokio::test]
async fn ink_multilayer_erc20() -> anyhow::Result<()> {
    const ALICE: AccountKeyring = AccountKeyring::Alice;
    const BOB: AccountKeyring = AccountKeyring::Bob;
    const EVE: AccountKeyring = AccountKeyring::Eve;

    let api = crate::API::from_url(
        std::env::var("END_POINT").unwrap_or_else(|_| "ws://127.0.0.1:9944".to_string()),
    )
    .await?;
    let mut contract = Contract::new("../contracts/native_token_wrapper.contract")?;

    // 1. Deploy the system-contract (native_token_wrapper)
    contract
        .deploy_as_system_contract(&api, None, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("create_wrapper_token", [format!("{}", 0_u32)])
                .unwrap()
        })
        .await?;

    // 2. Test name()
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("name", []).unwrap()
        })
        .await?;

    let name = <String>::decode(&mut rv.as_bytes_ref())?;
    // let name_rpc = CurrencyId::NativeToken(TokenId::Laguna).name();
    assert_eq!(name, "LAGUNA");

    // 3. Test symbol()
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("symbol", []).unwrap()
        })
        .await?;

    let symbol = <String>::decode(&mut rv.as_bytes_ref())?;
    // let symbol_rpc = CurrencyId::NativeToken(TokenId::Laguna).symbol();
    assert_eq!(symbol, "LAGUNA");

    // 4. Test decimals()
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("decimals", []).unwrap()
        })
        .await?;

    let decimals = <u8>::decode(&mut rv.as_bytes_ref())?;
    // let decimals_rpc = CurrencyId::NativeToken(TokenId::Laguna).decimals();
    assert_eq!(decimals, 18_u8);

    // 5. Test total_supply()
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("total_supply", []).unwrap()
        })
        .await?;

    let total_supply = <U256>::decode(&mut rv.as_bytes_ref())?;
    let total_issuance: U256 = async {
        let key = node::storage()
            .tokens()
            .total_issuance(CurrencyId::NativeToken(TokenId::Laguna));
        api.storage().fetch_or_default(&key, None).await
    }
    .await?
    .into();
    assert_eq!(total_supply, total_issuance);

    // 6. Test balance_of()
    let rv = contract
        .try_call(&api, ALICE, 0, &|t: ContractMessageTranscoder<'_>| {
            t.encode::<_, String>("balance_of", [format!("{:?}", EVE.to_account_id())])
                .unwrap()
        })
        .await?;

    let eve_balance = <U256>::decode(&mut rv.as_bytes_ref())?;
    let eve_balance_rpc: U256 = free_balance_of(&api, EVE.to_account_id()).await?.into();
    assert_eq!(eve_balance, eve_balance_rpc);

    // 7. Test transfer()
    // @dev: EVE transfers BOB 10 LAGUNA
    let value = U256::exp10(1 + 18);
    let sel_transfer = &|t: ContractMessageTranscoder<'_>| {
        t.encode::<_, String>(
            "transfer",
            [
                format!("{:?}", BOB.to_account_id()),
                format!("{:?}", value.0),
            ],
        )
        .unwrap()
    };

    let bob_balance_before: U256 = free_balance_of(&api, BOB.to_account_id()).await?.into();
    contract.call(&api, EVE, 0, sel_transfer).await?;

    let eve_balance_after: U256 = free_balance_of(&api, EVE.to_account_id()).await?.into();
    let bob_balance_after: U256 = free_balance_of(&api, BOB.to_account_id()).await?.into();

    // FIXME: need to obtain storage cost so we can compare the desired results more intuitively
    // assert_eq!(eve_balance_after, eve_balance_rpc - value - storage_fees); // fees adjusted
    assert_eq!(bob_balance_after, bob_balance_before + value);

    // 8. Test allowance(BOB, ALICE)
    let sel_allowance = &|t: ContractMessageTranscoder<'_>| {
        t.encode::<_, String>(
            "allowance",
            [
                format!("{:?}", BOB.to_account_id()),
                format!("{:?}", ALICE.to_account_id()),
            ],
        )
        .unwrap()
    };

    let rv = contract.try_call(&api, ALICE, 0, sel_allowance).await?;

    let allowance = <U256>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(allowance, U256::zero());

    // 9. Test approve()
    // @dev: BOB approves ALICE to spend upto 1 LAGUNA
    let allow_value = U256::exp10(18);
    let sel_approve = &|t: ContractMessageTranscoder<'_>| {
        t.encode::<_, String>(
            "approve",
            [
                format!("{:?}", ALICE.to_account_id()),
                format!("{:?}", allow_value.0),
            ],
        )
        .unwrap()
    };

    contract.call(&api, BOB, 0, sel_approve).await?;

    let rv = contract.try_call(&api, BOB, 0, sel_allowance).await?;

    let allowance = <U256>::decode(&mut rv.as_bytes_ref())?;
    assert_eq!(allowance, allow_value);

    // 10. Test transfer_from()
    // @dev: ALICE transfers 0.1 LAGUNA from BOB to EVE
    let transfer_value = U256::exp10(18 - 1);
    let sel_transfer_from = &|t: ContractMessageTranscoder<'_>| {
        t.encode::<_, String>(
            "transfer_from",
            [
                format!("{:?}", BOB.to_account_id()),
                format!("{:?}", EVE.to_account_id()),
                format!("{:?}", transfer_value.0),
            ],
        )
        .unwrap()
    };

    let bob_balance_before: U256 = free_balance_of(&api, BOB.to_account_id()).await?.into();
    let eve_balance_before: U256 = free_balance_of(&api, EVE.to_account_id()).await?.into();

    contract.call(&api, ALICE, 0, sel_transfer_from).await?;

    let bob_balance_after: U256 = free_balance_of(&api, BOB.to_account_id()).await?.into();
    let eve_balance_after: U256 = free_balance_of(&api, EVE.to_account_id()).await?.into();
    let rv = contract.try_call(&api, BOB, 0, sel_allowance).await?;

    let updated_allowance = <U256>::decode(&mut rv.as_bytes_ref())?;

    assert_eq!(bob_balance_after, bob_balance_before - transfer_value);
    assert_eq!(eve_balance_after, eve_balance_before + transfer_value);
    assert_eq!(updated_allowance, allowance - transfer_value);

    Ok(())
}
