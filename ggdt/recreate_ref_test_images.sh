#!/bin/sh
cd "$(dirname $"0")"
RUSTFLAGS="--cfg recreate_ref_test_images" cargo test
