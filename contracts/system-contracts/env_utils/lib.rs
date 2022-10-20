#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod env_utils {

    #[ink(storage)]
    pub struct EnvUtils {}

    impl EnvUtils {
        #[ink(constructor, selector = 0x861731d5)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message, selector = 0x649c07d5)]
        pub fn is_contract(&self, account: AccountId) -> bool {
            self.env().is_contract(&account)
        }

        #[ink(message, selector = 0x9804bf43)]
        pub fn code_hash(&self, account: AccountId) -> (bool, Hash) {
            match self.env().code_hash(&account) {
                Ok(h) => (true, h),
                Err(_) => (false, Hash::from([0u8; 32])),
            }
        }

        #[ink(message, selector = 0x4ed65189)]
        pub fn own_code_hash(&self) -> Hash {
            let caller = self.env().caller();
            assert!(self.is_contract(caller), "Caller is not a contract");
            self.code_hash(caller).1
        }

        #[ink(message, selector = 0x7143b598)]
        pub fn ecdsa_to_eth_address(&self, pubkey: [u8; 33]) -> (bool, [u8; 20]) {
            match self.env().ecdsa_to_eth_address(&pubkey) {
                Ok(addr) => (true, addr),
                Err(_) => (false, [0u8; 20]),
            }
        }

        #[ink(message, selector = 0xe1e0e895)]
        pub fn ecdsa_recover(
            &self,
            signature: [u8; 65],
            message_hash: [u8; 32],
        ) -> (bool, [u8; 33]) {
            match self.env().ecdsa_recover(&signature, &message_hash) {
                Ok(h) => (true, h),
                Err(_) => (false, [0u8; 33]),
            }
        }
    }
}
