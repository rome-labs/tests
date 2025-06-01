# Tests
This repository contains tests for the EVM, Rhea and Hercules projects.

## Integration tests 
1. To run integration tests, execute the following commands from the root directory:

For executing Uniswap tests:

```sh
./ci/scripts/uniswap_proxy.sh
```

For executing OpenZeppelin tests:

```sh
./ci/scripts/open_zeppelin_proxy.sh
```

For executing Rome tests:

```sh
./ci/scripts/evm.sh
```

These commands will generate a set of logs in `records` folder, including:

- Uniswap Tests for proxy and Op-Geth
- OpenZeppelin tests for proxy and Op-Geth 

1. To generate the CTRF, execute the following command:

```sh
./ci/scripts/generate_ctrf.sh 
```

The generated CTRF will be available inside `records/ctrf.json`.

