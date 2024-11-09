# tests
tests of the evm, rhea

## Integration tests 

1. To run integration tests simply execute the following from root

For executing uniswap tests:-

```
./scripts/uniswap_proxy.sh
```

For executing open-zeppelin tests:-

```
./scripts/open_zeppelin_proxy.sh
```

For rome tests:- 
```
./scripts/evm.sh
```

This will dump a set of logs for each test suite namely:- 

- Uniswap Tests for proxy and Op-Geth 
- Open Zeppelin tests for proxy and Op-Geth 

2. To generate CTRF execute the following

```
./scripts/generate_ctrf.sh 
```

The executed CTRF will be available inside "records/ctrf.json"
