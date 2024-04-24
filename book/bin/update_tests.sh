#!/bin/bash

set -eou pipefail

BASEDIR=$(dirname "$0")/..

cd "$BASEDIR"

cat > book_tests/src/doctests.rs <<EOF
// Automatically generated. Do not edit.
#![allow(non_snake_case)]

EOF

for mdfile in $(find ./src -name '*.md'); do

    ident=${mdfile#./src/}
    ident=${ident%.md}
    ident=${ident//-/_}

    cat >>book_tests/src/doctests.rs <<EOF
#[doc = include_str!("../../$mdfile")]
mod $ident {}
EOF

done
