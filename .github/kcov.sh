#!/bin/bash

REPORT=$(find ./target/debug -maxdepth 2 -regex '.+/deps/.*' -a ! -regex '.+\.\(d\|rlib\|rmeta\|so\)')
for file in $REPORT; do
  echo $file
  kcov --include-pattern=vicis_ir/src --exclude-pattern=/.cargo ./target/cov "$file"
  bash <(curl -s https://codecov.io/bash) -s ./target/cov
done
