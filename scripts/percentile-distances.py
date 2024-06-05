#! /usr/bin/env python3

import argparse

import numpy as np
import pandas as pd

import util


def main():
    parser = argparse.ArgumentParser(
        description="Get the percentile distances of a dataset from a distance matrix"
    )
    parser.add_argument(
        "input_file", type=str, help="input file containing the distance matrix"
    )
    parser.add_argument(
        "percentiles", type=float, nargs="+", help="percentiles to calculate"
    )
    args = parser.parse_args()
    df = util.read_distance_matrix(args.input_file)
    v = df.values.flatten()

    for distance in np.percentile(v, args.percentiles):
        print(distance)


if __name__ == "__main__":
    main()
