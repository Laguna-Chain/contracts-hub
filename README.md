# contracts hub

This repository is used to show case how to build smart contracts on laguna-chain-devnet. It also contains a integration tests with the node to ensure that all examples are compatible on chain.

## prepare contracts

laguna-chain uses pallet-contracts under the hood, which means you'll be able to use all of the supported source langues `ink`, `solang` or `ask` that can be compiled in wasm. 

### compile solidity contracts with solang

You'll need to first obtain the solang compiler, which should be available for you if you can access laguna-chain's private docker registry:

```bash
docker run --rm -it us-docker.pkg.dev/laguna-chain/laguna-chain/solang:ink_metadata --help
```

> this will be fixed once it's merged upstream, and eventually you can get the official binary and run:
>
> solang compile $ARGS

**compile solidity contracts**

To compile a contract, use this command:

```bash 
docker run --rm -it -v $(pwd)/$HOST_DIR:/$MOUNT_PATH_IN_CONTAINER us-docker.pkg.dev/laguna-chain/laguna-chain/solang:ink_metadata compile --target substrate $MOUNT_PATH_IN_CONTAINER/$CONTRACT_PATH.sol -o $MOUNT_PATH_IN_CONTAINER/$OUTPUT_PATH
```

Afer this you'll recieve two file at your specified path:
`$CONTRACT.contract`, and `$CONTRACT.wasm`, which the first one is the ABI bundle with the wasm code inside it. And a standalone wasm file.


## interact with contracts through a front-end

Once you have prepared your ABI bundle and wasm blob, you can use either of the methods to deploy to laguna-chain-devnet:
1. polkadot.js/apps contracts tab(https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Flaguna-chain-dev.hydrogenx.tk%3A443#/contracts) 
2. use contracts-ui(https://contracts-ui.substrate.io/?rpc=wss://laguna-chain-dev.hydrogenx.tk:443)

## interact with contracts through a cli

> TBD: javscript with polkadot.js
>
> TBD: rust with subxt

---

## integration tests

Currently a subxt integration tests is maintained under this repo with the package [`subxt-tests`](./subxt-tests).

The test can be run at the subxt directory:

```bash
cd subxt-tests
END_POINT=$WS_ENDPOINT_DEV_NET cargo test -- --test-threads=1
```






