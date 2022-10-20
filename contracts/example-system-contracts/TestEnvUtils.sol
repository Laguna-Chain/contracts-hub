// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;
import "../system-contracts/env_utils/interface.sol";

contract TestEnvUtils {
    EnvUtils sys_contract;

    constructor(address sys_contract_addr) {
        sys_contract = EnvUtils(sys_contract_addr);
    }

    function is_contract(address account) external view returns(bool) {
        return sys_contract.is_contract(account);
    }

    function code_hash(address account) external view returns(bool, bytes32) {
        return sys_contract.code_hash(account);
    }

    function own_code_hash() external view returns(bytes32) {
        return sys_contract.own_code_hash();
    }

    function ecdsa_to_eth_address(uint8[33] calldata pubkey) external view returns(bool, bytes20) {
        return sys_contract.ecdsa_to_eth_address(pubkey);
    }

    function ecdsa_recover(uint8[65] calldata signature, bytes32 message_hash) external view returns(bool, uint8[33] memory) {
        return sys_contract.ecdsa_recover(signature, message_hash);
    }
}
