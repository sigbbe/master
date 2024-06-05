#! /usr/bin/env python3

import json
import os
import subprocess
import sys
from itertools import product

from tqdm import tqdm

DYFT_CONFIGS = {
    "bits": 8,
    "errors": [0, 4, 8],
    "radius": [0, 4, 60],
    "in_weight": [0.1, 1.0, 5.0],
    "l": 8,
    "k": 2,
    "resolution": [0.0029875, 0.005975, 0.01195],
    "seed": 1,
}

FRESH_CONFIGS = {
    "resolution": [0.0029875, 0.005975, 0.01195],
    "k": 2,
    "l": [8, 16, 32, 64],
    "verify_fraction": [0.0, 0.25, 0.5],
    "seed": 1,
}

DATASET_PATH = "porto-5000.parquet"
QUERYSET_PATH = "porto-query-1.parquet"
DYFT = "./target/release/dyft"
FRESH = "./target/release/fresh"

RUN_DYFT = True


def generate_configs(configs):
    keys = configs.keys()
    values = configs.values()
    fixed_values = {
        key: value for key, value in configs.items() if not isinstance(value, list)
    }
    variable_values = {
        key: value for key, value in configs.items() if isinstance(value, list)
    }
    combinations = [
        dict(zip(variable_values.keys(), values))
        for values in product(*variable_values.values())
    ]
    return [{**fixed_values, **combination} for combination in combinations]


def create_fresh_config(
    resolution, k, l, seed, n=None, verify_fraction=None, distance=None
):
    return {
        "resolution": resolution,
        "k": k,
        "l": l,
        "verify_fraction": verify_fraction,
        "distance": distance,
        "seed": seed,
        "n": n,
    }


def create_dyft_config(
    bits, errors, radius, in_weight, l, k, resolution, seed, n=None, distance=None
):
    return {
        "bits": bits,
        "errors": errors,
        "radius": radius,
        "in_weight": in_weight,
        "l": l,
        "k": k,
        "resolution": resolution,
        "seed": seed,
        "distance": distance,
        "n": n,
    }


def query_dyft_cmd(config, dataset, queryset, samples=None):
    """
    dyft query
        --bits <BITS>
        --errors <ERRORS>
        --radius <RADIUS>
        --in_weight <IN_WEIGHT>     // optional (default: 1.0)
        --distance <DISTANCE>       // optional (default: None)
        -k <K>
        -l <L>
        --resolution <RESOLUTION>
        --seed <SEED>
        --datapath <DATAPATH>
        --querypath <QUERYPATH>
    """
    return (
        [
            DYFT,
            "build" if queryset is None else "query",
            "--bits",
            str(config["bits"]),
            "--errors",
            str(config["errors"]),
            "--radius",
            str(config["radius"]),
            "--in-weight",
            str(config["in_weight"]),
            "--k",
            str(config["k"]),
            "--l",
            str(config["l"]),
            "--resolution",
            str(config["resolution"]),
            "--seed",
            str(config["seed"]),
            "--dataset",
            dataset,
        ]
        + ([] if queryset is None else ["--queryset", queryset])
        + (
            []
            if config["distance"] is None
            else ["--distance", str(config["distance"])]
        )
        + ([] if config["n"] is None else ["--n", str(config["n"])])
        + [str(s) for s in samples]
        if samples is not None
        else []
    )


def query_fresh_cmd(config, dataset, queryset):
    """
    fresh query
        -k <K>
        -l <L>
        --resolution <RESOLUTION>
        --seed <SEED>
        --datapath <DATAPATH>
        --querypath <QUERYPATH>
    """
    return (
        [
            FRESH,
            "build" if queryset is None else "query",
            "--k",
            str(config["k"]),
            "--l",
            str(config["l"]),
            "--resolution",
            str(config["resolution"]),
            "--seed",
            str(config["seed"]),
            "--dataset",
            dataset,
        ]
        + ([] if queryset is None else ["--queryset", queryset])
        + (
            [
                "--distance",
                str(config["distance"]),
                "--verify-fraction",
                str(config["verify_fraction"]),
            ]
            if config["distance"] is not None and config["verify_fraction"] is not None
            else []
        )
        + ([] if config["n"] is None else ["-n", str(config["n"])])
    )


def run_program(command):
    # print(" ".join(command))
    # return {}
    result = subprocess.run(command, capture_output=True, text=False, check=True)
    try:
        return json.loads(result.stdout) if result.check_returncode() == None else None
    except subprocess.CalledProcessError as e:
        return None


import argparse


def parse_fuzz_params_args():
    parser = argparse.ArgumentParser(description="Fuzz parameters for dyft")
    parser.add_argument(
        "index", choices=["dyft", "fresh"], help="Choose either 'dyft' or 'fresh'"
    )
    parser.add_argument(
        "--dataset",
        type=str,
    )
    parser.add_argument(
        "--queryset",
        type=str,
    )
    parser.add_argument(
        "--n", type=int, nargs="*", help="Number of points in the dataset"
    )
    parser.add_argument("--bits", type=int, nargs="*", help="Number of bits for dyft")
    parser.add_argument(
        "--errors", type=int, nargs="*", help="Number of errors for dyft"
    )
    parser.add_argument("--radius", type=int, nargs="*", help="Radius for dyft")
    parser.add_argument("--in-weight", type=float, nargs="*", help="In weight for dyft")
    # LSH
    parser.add_argument(
        "--delta",
        type=float,
        help="Resolution of the grid to fuzz",
    )
    parser.add_argument("--l", type=int, nargs="*", help="Number of hash functions")
    parser.add_argument(
        "--k", type=int, nargs="*", help="Number of hash concatenations"
    )
    parser.add_argument("--seed", type=int, nargs="*", help="Random seed")
    # Both
    parser.add_argument(
        "--distance", type=float, nargs="*", help="Distance threshold for fresh"
    )
    # Fresh
    parser.add_argument(
        "--verify_fraction", type=float, nargs="*", help="Verify fraction for fresh"
    )

    try:
        args = parser.parse_args()
        args.bits = args.bits if args.bits is not None else 8
        args.errors = args.errors if args.errors is not None else 8
        args.radius = args.radius if args.radius is not None else 8
        args.in_weight = args.in_weight if args.in_weight is not None else 1.0
        args.l = args.l if args.l is not None else 8
        args.k = args.k if args.k is not None else 2
        args.seed = args.seed if args.seed is not None else 0
        if args.n is not None:
            if isinstance(args.n, list):
                args.n = [n if n != 0 else None for n in args.n]
        return args
    except argparse.ArgumentError as e:
        parser.print_help()
        sys.exit(1)


def main():
    params = parse_fuzz_params_args()
    if params.index == "dyft":
        dyft_configs = generate_configs(
            create_dyft_config(
                bits=params.bits,
                errors=params.errors,
                radius=params.radius,
                in_weight=params.in_weight,
                l=params.l,
                k=params.k,
                resolution=params.delta * 8,
                seed=params.seed,
            )
        )
        results = [
            run_program(
                query_dyft_cmd(
                    config,
                    dataset=params.dataset,
                    queryset=params.queryset,
                    samples=params.n,
                )
            )
            for config in tqdm(dyft_configs)
        ]
        print(json.dumps(results))
    elif params.index == "fresh":
        fresh_config = create_fresh_config(
            resolution=params.delta * 8,
            k=params.k,
            l=params.l,
            seed=params.seed,
            verify_fraction=params.verify_fraction,
            n=params.n,
            distance=params.delta,
        )
        configs = generate_configs(fresh_config)
        results = [
            run_program(
                query_fresh_cmd(
                    config, dataset=params.dataset, queryset=params.queryset
                )
            )
            for config in tqdm(configs)
        ]
        print(json.dumps(results))


if __name__ == "__main__":
    main()