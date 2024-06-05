#! /usr/bin/env python3

import os
import sys
import warnings

import numpy as np
import pandas as pd

import util

warnings.filterwarnings("ignore")


def extract_trajectories_args():
    import argparse

    parser = argparse.ArgumentParser(
        description="split a trajectory dataset into data and query sets"
    )
    parser.add_argument(
        "input_file",
        type=str,
        help="input file containing the trajectory dataset",
    )
    parser.add_argument(
        "--seed",
        type=int,
        help="seed for random number generator",
        default=42,
    )
    parser.add_argument(
        "-o",
        "--output",
        type=str,
        help="output file containing the query set",
    )
    parser.add_argument(
        "-n",
        "--n-trajectories",
        type=int,
        default=1000,
        help="number of trajectories to extract from the dataset",
    )
    parser.add_argument(
        "--query",
        action="store_true",
        help="Extract query set instead of data set",
    )
    return parser.parse_args()


def main():
    args = extract_trajectories_args()
    rand = np.random.default_rng(args.seed)
    df = pd.read_parquet(util.dataset_file_path(args.input_file))
    index = df.index.unique()
    index = rand.choice(index, args.n_trajectories, replace=False)
    extract = df.loc[index]
    extract.to_parquet(
        util.query_file_path(args.output)
        if args.query
        else util.dataset_file_path(args.output)
    )
    df.drop(index, inplace=True)
    df.to_parquet(util.dataset_file_path(args.input_file))
    return 0


if __name__ == "__main__":
    sys.exit(main())
