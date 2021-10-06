# Benchmarking
1. Use the following command to generate weights.rs
```
./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet_template --extrinsic do_something --steps 50 --repeat 20 --output pallets/template/src/weights.rs --template .maintain/frame-weight-template.hbs
```
2. Modify the weight info in pallet. For details please see this commit change https://github.com/h4n0/substrate-course-tasks/commit/3c82b53f611037dba6db3ee84aba98fe45968435#diff-5950c339250b57ceb0f4df93f1847a4c5f9b21fd85500501e5a4541d9bc056e6


# Chainspecs
1. The generated chain specs are located https://github.com/h4n0/substrate-course-tasks/tree/main/8_benchmark/chain_spec
2. The commands used for generating chain specs are:
```
./target/release/node-template build-spec --disable-default-bootnode --chain dev > chain_spec/dev_spec.json
./target/release/node-template build-spec --disable-default-bootnode --chain=chain_spec/dev_spec.json --raw > chain_spec/dev_spec-raw.json
```
