#!/usr/bin/env bash
# 构建试点队列测试/基准。用法: bash build.sh
set -euo pipefail
cd "$(dirname "$0")"
CXX="${CXX:-g++}"
mkdir -p build
"$CXX" -std=c++17 -O2 -Wall -Wextra -pthread -Isrc tests/test_queue.cpp -o build/test_queue.exe
echo "build ok -> build/test_queue.exe"
