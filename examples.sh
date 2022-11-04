#!/bin/bash
for ex in examples/*.rs; do
    ex=${ex/.rs/}
    ex=${ex/examples\//}
    echo "Running $ex"
    cargo run --example $ex
done