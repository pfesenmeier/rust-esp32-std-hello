#!/bin/bash

./build.sh && espflash.exe COM6 target/riscv32imc-esp-espidf/debug/main_copy && espmonitor.exe COM6
