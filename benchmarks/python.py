#!/usr/bin/env python3
"""Equivalent checked-I64 Core workloads, using Python integers."""

import json
import sys
import time

MIN_I64 = -(1 << 63)
MAX_I64 = (1 << 63) - 1
MASK_U64 = (1 << 64) - 1


def checked(value):
    if not MIN_I64 <= value <= MAX_I64:
        raise OverflowError("I64 overflow")
    return value


def increment(x):
    if x >= MAX_I64:
        raise ValueError("precondition")
    result = checked(x + 1)
    if result != checked(x + 1):
        raise ValueError("postcondition")
    return result


def absolute(x):
    if x == MIN_I64:
        raise ValueError("precondition")
    result = checked(-x) if x < 0 else x
    if result < 0:
        raise ValueError("postcondition")
    return result


def batch(function, count):
    checksum = 0
    for i in range(count):
        checksum = (checksum + function((i & 2047) - 1024)) & MASK_U64
    return checksum


def self_test():
    for x in (MIN_I64, MIN_I64 + 1, -1024, -1, 0, 1, 1023, MAX_I64 - 1, MAX_I64):
        if x != MAX_I64 and increment(x) != x + 1:
            raise RuntimeError("incorrect increment")
        if x != MIN_I64 and absolute(x) != abs(x):
            raise RuntimeError("incorrect absolute")
    for function, x, error in (
        (increment, MAX_I64, ValueError),
        (absolute, MIN_I64, ValueError),
        (checked, MAX_I64 + 1, OverflowError),
        (checked, MIN_I64 - 1, OverflowError),
    ):
        try:
            function(x)
        except error:
            continue
        raise RuntimeError("missing contract/overflow failure")
    print("self-test passed")


def main():
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return
    if len(sys.argv) != 4 or sys.argv[1] not in ("increment", "absolute"):
        raise SystemExit("usage: python.py increment|absolute ITERATIONS WARMUP_ITERATIONS")
    count, warmup = map(int, sys.argv[2:])
    if not 1 <= count <= 1_000_000_000 or not 0 <= warmup <= 1_000_000_000:
        raise SystemExit("invalid iteration counts")
    function = increment if sys.argv[1] == "increment" else absolute
    batch(function, warmup)
    start = time.perf_counter_ns()
    checksum = batch(function, count)
    elapsed = time.perf_counter_ns() - start
    print(json.dumps({"ns_per_call": elapsed / count, "elapsed_ns": str(elapsed),
                      "checksum": str(checksum), "iterations": count}))


if __name__ == "__main__":
    main()
