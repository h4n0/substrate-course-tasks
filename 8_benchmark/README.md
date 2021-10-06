# Benchmarking
1. Use the following command to generate weights.rs
```
./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet_template --extrinsic do_something --steps 50 --repeat 20 --output pallets/template/src/weights.rs --template .maintain/frame-weight-template.hbs
```
2. Modify the weight info in pallet. For details please see this commit change https://github.com/h4n0/substrate-course-tasks/commit/3c82b53f611037dba6db3ee84aba98fe45968435#diff-5950c339250b57ceb0f4df93f1847a4c5f9b21fd85500501e5a4541d9bc056e6


