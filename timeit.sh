#!/usr/bin/env bash

cargo run --release | tail -n2 -- > stats.csv
for i in {2..100}
do
  cargo run --release | tail -n1 -- >> stats.csv
done
