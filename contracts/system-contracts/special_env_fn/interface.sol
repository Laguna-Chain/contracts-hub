// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface special_env_fn {
    function is_contract(address account) external view returns(bool);
    function code_hash(address account) external view returns(bool, bytes32);
    function own_code_hash() external view returns(bytes32);
    function ecdsa_to_eth_address(uint8[33] calldata pubkey) external view returns(bool, bytes20);
    function ecdsa_recover(uint8[65] calldata signature, bytes32 message_hash) external view returns(bool, uint8[33] memory);
}
