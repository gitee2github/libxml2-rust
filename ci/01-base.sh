#!/bin/bash

# cd ..

# cd ./libxml2-2.9.12_github_version
rm -rf CMakeCache.txt
rm -rf cmake_install.cmake
rm -rf CMakeFiles
rm -rf Makefile
cmake -DSTEP="build"
make
cp libxml2.a ./libxml2_static.a

cd ./rust

#开始检查
cargo fmt --all -- --check -v
cargo clean

#cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo clippy --all-targets --all-features --tests --benches -- -v
cargo clean

cargo check
cargo clean

# cargo build
cargo build --release -v
cd ../
cmake -DSTEP="link"
make 
ctest

#cargo rustc -- -D warnings
# bin=$(sed -n '/[[bin]]/ {n;p}' Cargo.toml | sed 's/\"//g' | sed 's/name = //g')
# for bin_name in $bin
# do
# echo $bin_name
# cargo rustc --bin $bin_name -- -D warnings -v
# done

# cargo build --release -v

#RUST_BACKTRACE=1 cargo test --all -v -- --nocapture --test-threads=1
# RUST_BACKTRACE=1 cargo test --all -- --nocapture

# cargo doc --all --no-deps
