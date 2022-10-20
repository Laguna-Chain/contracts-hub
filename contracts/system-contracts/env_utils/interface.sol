// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface EnvUtils {
    /// Checks whether the specified account is a contract.
    function is_contract(address account) external view returns(bool);

    /// Retrieves the code hash of the contract at the specified account (if it exists)
    function code_hash(address account) external view returns(bool, bytes32);

    /// Retrieves the code hash of the currently executing contract.
    function own_code_hash() external view returns(bytes32);

    /// Returns an Ethereum address from the ECDSA compressed public key 
    /// if valid pubkey provided else returns (false, bytes20(0))
    function ecdsa_to_eth_address(uint8[33] calldata pubkey) external view returns(bool, bytes20);

    /// Recovers the compressed ECDSA public key for given signature and message_hash
    /// Incase of error, (false, bytes33(0)) is returned
    function ecdsa_recover(uint8[65] calldata signature, bytes32 message_hash) external view returns(bool, uint8[33] memory);
}
