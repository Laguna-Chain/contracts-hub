use std::marker::PhantomData;

use crate::{node, API};
use parity_scale_codec::Encode;

mod cases;
mod rpc;

use ethereum::{
    TransactionAction, TransactionSignature, TransactionV2 as Transaction,
    {EIP1559TransactionMessage, EIP2930TransactionMessage, LegacyTransactionMessage},
};

use sp_core::{ecdsa, serde::Serialize, Bytes, Pair, H160, H256, U256};
use subxt::rpc::{rpc_params, ClientT};

use crate::generic_client::load_project;
use crate::utils::free_balance_of;

pub struct TxWrapper<T> {
    _marker: PhantomData<T>,
}

pub trait SignPayload {
    type Payload;

    fn sign(payload: Self::Payload, private_key: &H256) -> Transaction;
}

pub trait PayloadFactory {
    type Payload;

    fn create(value: U256, code: Bytes, selector: Bytes, salt: Bytes) -> Self::Payload;
    fn call(target: H160, value: U256, input: Bytes) -> Self::Payload;
    fn transfer(target: H160, value: U256) -> Self::Payload;

    fn update_defaults(api: &API, payload: &mut Self::Payload);
}

impl PayloadFactory for TxWrapper<LegacyTransactionMessage> {
    type Payload = LegacyTransactionMessage;

    fn create(value: U256, code: Bytes, selector: Bytes, salt: Bytes) -> Self::Payload {
        let input = (code, selector, salt).encode();

        LegacyTransactionMessage {
            action: TransactionAction::Create,
            value,
            input,
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
        }
    }

    fn call(target: H160, value: U256, input: Bytes) -> Self::Payload {
        LegacyTransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: input.to_vec(),
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
        }
    }

    fn transfer(target: H160, value: U256) -> Self::Payload {
        LegacyTransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: Default::default(),
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
        }
    }

    fn update_defaults(api: &API, payload: &mut Self::Payload) {
        let fut = async move { api.rpc() };

        todo!()
    }
}

impl PayloadFactory for TxWrapper<EIP2930TransactionMessage> {
    type Payload = EIP2930TransactionMessage;

    fn create(value: U256, code: Bytes, selector: Bytes, salt: Bytes) -> Self::Payload {
        let input = (code, selector, salt).encode();

        EIP2930TransactionMessage {
            action: TransactionAction::Create,
            value,
            input,
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
        }
    }

    fn call(target: H160, value: U256, input: Bytes) -> Self::Payload {
        EIP2930TransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: input.to_vec(),
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
        }
    }

    fn transfer(target: H160, value: U256) -> Self::Payload {
        EIP2930TransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: Default::default(),
            nonce: Default::default(),
            gas_price: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
        }
    }

    fn update_defaults(api: &API, payload: &mut Self::Payload) {
        todo!()
    }
}

impl PayloadFactory for TxWrapper<EIP1559TransactionMessage> {
    type Payload = EIP1559TransactionMessage;

    fn create(value: U256, code: Bytes, selector: Bytes, salt: Bytes) -> Self::Payload {
        let input = (code, selector, salt).encode();

        EIP1559TransactionMessage {
            action: TransactionAction::Create,
            value,
            input,
            nonce: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
        }
    }

    fn call(target: H160, value: U256, input: Bytes) -> Self::Payload {
        EIP1559TransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: input.to_vec(),
            nonce: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
        }
    }

    fn transfer(target: H160, value: U256) -> Self::Payload {
        EIP1559TransactionMessage {
            action: TransactionAction::Call(target),
            value,
            input: Default::default(),
            nonce: Default::default(),
            chain_id: Default::default(),
            gas_limit: Default::default(),
            access_list: Default::default(),
            max_priority_fee_per_gas: Default::default(),
            max_fee_per_gas: Default::default(),
        }
    }

    fn update_defaults(api: &API, payload: &mut Self::Payload) {
        todo!()
    }
}

impl SignPayload for TxWrapper<LegacyTransactionMessage> {
    type Payload = LegacyTransactionMessage;

    fn sign(payload: Self::Payload, private_key: &H256) -> Transaction {
        let pair = ecdsa::Pair::from_seed(&private_key.0);
        let hash = payload.hash();

        let s = pair.sign_prehashed(&hash.0);

        let sig = &s.0[0..64];

        // recovery_id is the last byte of the signature
        let recid = &s.0[64];

        let chain_id = payload
            .chain_id
            .expect("please speficy the correct chain_id");

        let sig = TransactionSignature::new(
            *recid as u64 % 2 + chain_id * 2 + 35,
            H256::from_slice(&sig[0..32]),
            H256::from_slice(&sig[32..64]),
        )
        .unwrap();

        Transaction::Legacy(ethereum::LegacyTransaction {
            nonce: payload.nonce,
            gas_price: payload.gas_price,
            gas_limit: payload.gas_limit,
            action: payload.action,
            value: payload.value,
            input: payload.input,
            signature: sig,
        })
    }
}

impl SignPayload for TxWrapper<EIP2930TransactionMessage> {
    type Payload = EIP2930TransactionMessage;

    fn sign(payload: Self::Payload, private_key: &H256) -> Transaction {
        let pair = ecdsa::Pair::from_seed(&private_key.0);

        let hash = payload.hash();

        let s = pair.sign_prehashed(&hash.0);

        let sig = &s.0[0..64];

        // recovery_id is the last byte of the signature
        let recid = &s.0[64];

        let r = H256::from_slice(&sig[0..32]);
        let s = H256::from_slice(&sig[32..64]);

        Transaction::EIP2930(ethereum::EIP2930Transaction {
            chain_id: payload.chain_id,
            nonce: payload.nonce,
            gas_price: payload.gas_price,
            gas_limit: payload.gas_limit,
            action: payload.action,
            value: payload.value,
            input: payload.input.clone(),
            access_list: payload.access_list,
            odd_y_parity: *recid != 0,
            r,
            s,
        })
    }
}

impl SignPayload for TxWrapper<EIP1559TransactionMessage> {
    type Payload = EIP1559TransactionMessage;

    fn sign(payload: Self::Payload, private_key: &H256) -> Transaction {
        let pair = ecdsa::Pair::from_seed(&private_key.0);

        let hash = payload.hash();

        let s = pair.sign_prehashed(&hash.0);

        let sig = &s.0[0..64];

        // recovery_id is the last byte of the signature
        let recid = &s.0[64];

        let r = H256::from_slice(&sig[0..32]);
        let s = H256::from_slice(&sig[32..64]);
        Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: payload.chain_id,
            nonce: payload.nonce,
            max_priority_fee_per_gas: payload.max_priority_fee_per_gas,
            max_fee_per_gas: payload.max_fee_per_gas,
            gas_limit: payload.gas_limit,
            action: payload.action,
            value: payload.value,
            input: payload.input.clone(),
            access_list: payload.access_list,
            odd_y_parity: *recid != 0,
            r,
            s,
        })
    }
}
