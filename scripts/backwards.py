#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import io
import os
import re
import sys
import threading
import zipfile
from ast import literal_eval

import numpy as np
import pandas as pd
from tqdm import tqdm

pd.options.mode.copy_on_write = True
from util import (
    ID_COLUMN,
    LAT_COLUMN,
    LON_COLUMN,
    filter_distance,
    in_file,
    out_file,
    preprocess_args,
)

"""
Each trajectory consists of 721 datapoints, so to preprocess a certain amount of the data, set the nrows parameter to a multiplication of 721.
"""

INFILE = in_file("backward-trajectories.zip")
OUTFILE = out_file("backward.parquet")

BACKWARD_TRAJECTORY_LEN = 721
BACKWARD_COL = ["time", "lon", "lat", "z", "P", "QV", "T", "HSURF", "MODEL"]
RAND_ID = np.random.default_rng(42)

result = pd.DataFrame()


def process_backward_data(
    df: pd.DataFrame, distance_threshold: float, min_points: int, max_points: int
):
    global result

    # Manually set the column names
    df.columns = BACKWARD_COL

    # Take only the lat and lon columns
    df = df[[LAT_COLUMN, LON_COLUMN]]

    # Generate a random ID for each trajectory
    df.loc[:, ID_COLUMN] = np.array(
        [
            [RAND_ID.integers(low=0, high=2**63 - 1)] * BACKWARD_TRAJECTORY_LEN
            for _ in range(0, int(len(df) / BACKWARD_TRAJECTORY_LEN))
        ]
    ).flatten()

    # Set the index to the ID column
    df.set_index(ID_COLUMN, inplace=True)
    result = pd.concat([result, df])


def zip_file_iter(zip_file, nrows=None):
    with zipfile.ZipFile(zip_file) as z:
        files = z.namelist() if nrows is None else z.namelist()[:nrows]
        for filename in files:
            if not z.getinfo(filename).is_dir():
                with z.open(filename, "r") as f:
                    yield pd.read_csv(f, header=None, sep=r"\s+", skiprows=4)


def main(nrows, distance_threshold, min_points, max_points, save):
    # initialize a thread pool
    pool = []
    for file in tqdm(zip_file_iter(INFILE, nrows)):
        thread = threading.Thread(
            target=process_backward_data,
            args=(file, distance_threshold, min_points, max_points),
        )
        thread.start()
        pool.append(thread)

    # wait for all threads to finish
    [t.join() for t in pool]

    if save:
        result.to_parquet(OUTFILE)


if __name__ == "__main__":
    args = preprocess_args()
    main(
        None if args.all else args.nrows,
        args.distance_threshold,
        args.min_points,
        args.max_points,
        args.save,
    )
