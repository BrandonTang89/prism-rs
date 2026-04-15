#!/usr/bin/env python3

from __future__ import annotations

import argparse
import shlex
import subprocess
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Benchmark:
    name: str
    model_path: str
    const_overrides: str
    props: str
    runs: int | None = None
    warmup_runs: int = 1

    def command(self, binary_path: str, prop_path: str) -> str:
        args = [
            binary_path,
            "--model-type",
            "dtmc",
            "--model",
            self.model_path,
            "--prop-file",
            prop_path,
            "--props",
            self.props,
            "--const",
            self.const_overrides,
        ]
        return shlex.join(args)


BENCHMARKS: tuple[Benchmark, ...] = (
    Benchmark(
        name="leader5_6_check",
        model_path="tests/dtmc/leader5_6.prism",
        const_overrides="L=3",
        props="1,2",
        runs=3,
    ),
    Benchmark(
        name="leader5_7_check",
        model_path="tests/dtmc/leader5_7.prism",
        const_overrides="L=3",
        props="1,2",
        runs=3,
    ),
    Benchmark(
        name="leader6_6_check",
        model_path="tests/dtmc/leader6_6.prism",
        const_overrides="L=3",
        props="1,2",
        runs=3,
    ),
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run leader model-checking benchmarks with hyperfine."
    )
    parser.add_argument(
        "--binary",
        default="target/release/prism-rs",
        help="Path to prism-rs binary (default: target/release/prism-rs).",
    )
    parser.add_argument(
        "--prop-file",
        default="tests/dtmc/leader.prop",
        help="Property file for leader models (default: tests/dtmc/leader.prop).",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Skip cargo build --release before benchmarking.",
    )
    parser.add_argument(
        "--export-json",
        default="target/hyperfine-leader-checking.json",
        help=(
            "Base output path for hyperfine JSON results. "
            "One file per benchmark is written."
        ),
    )
    return parser.parse_args()


def ensure_binary(binary_path: str, skip_build: bool) -> None:
    if skip_build:
        return
    subprocess.run(["cargo", "build", "--release"], check=True)
    if not Path(binary_path).exists():
        raise FileNotFoundError(f"Expected binary at {binary_path}")


def json_path_for_benchmark(base_path: str, benchmark_name: str) -> str:
    base = Path(base_path)
    return str(base.with_name(f"{base.stem}-{benchmark_name}{base.suffix}"))


def run_hyperfine(binary_path: str, prop_path: str, export_json_base: str) -> None:
    for benchmark in BENCHMARKS:
        command: list[str] = [
            "hyperfine",
            "--warmup",
            str(benchmark.warmup_runs),
            "--export-json",
            json_path_for_benchmark(export_json_base, benchmark.name),
            "-n",
            benchmark.name,
            benchmark.command(binary_path, prop_path),
        ]

        if benchmark.runs is not None:
            command[1:1] = ["--runs", str(benchmark.runs)]

        subprocess.run(command, check=True)


def main() -> None:
    args = parse_args()
    ensure_binary(binary_path=args.binary, skip_build=args.skip_build)
    run_hyperfine(
        binary_path=args.binary,
        prop_path=args.prop_file,
        export_json_base=args.export_json,
    )


if __name__ == "__main__":
    main()
