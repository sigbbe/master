#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import concurrent.futures
import os
import sys
import threading
import warnings
import zipfile
from ast import literal_eval

import numpy as np
import pandas as pd
from tqdm import tqdm

warnings.filterwarnings("ignore")
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

PORTO_ID = "TRIP_ID"
PORTO_POLYLINE = "POLYLINE"
PORTO_MISSING_DATA = "MISSING_DATA"

INFILE = in_file("porto.zip")
OUTFILE = out_file("porto.parquet")


def drop_duplicates(df):
    # remove duplicate points
    df.drop_duplicates(
        subset=[ID_COLUMN, LAT_COLUMN, LON_COLUMN], keep="first", inplace=True
    )


def main(nrows=None, km=1.0, min_points=10, max_points=5000, save=False):
    df = pd.read_csv(INFILE, nrows=nrows)
    n = df.shape[0]

    # remove rows where the column MISSING_DATA is True
    df = df[df[PORTO_MISSING_DATA] == False]
    print(f"Removed {n - df.shape[0]} rows with missing data")

    # take only TRIP_ID and POLYLINE columns
    df = df[[PORTO_ID, PORTO_POLYLINE]]

    # transform the column POLYLINE
    df[PORTO_POLYLINE] = df[PORTO_POLYLINE].apply(lambda x: literal_eval(x)).dropna()
    df = df.explode(PORTO_POLYLINE)
    print(f"dataset contains {df.shape[0]} points")

    # rename id column
    df.rename(columns={PORTO_ID: ID_COLUMN}, inplace=True)

    # set the id column as index
    df.set_index(ID_COLUMN, inplace=True)

    # drop rows with missing values
    df.dropna(inplace=True)

    points = np.array([x for x in df[PORTO_POLYLINE].to_numpy()])
    df[LAT_COLUMN] = points[:, 1]
    df[LON_COLUMN] = points[:, 0]

    # drop POLYLINE column
    df.drop(columns=[PORTO_POLYLINE], inplace=True)

    df = filter_distance(df, km, min_points, max_points)

    if save:
        # save proccessed data to parquet
        print(df.head(20))

        print(f"Saving {df.index.value_counts().count()} curves to {OUTFILE}")

        df.to_parquet(OUTFILE, index=True)

    return 0


if __name__ == "__main__":
    args = preprocess_args()
    main(
        args.nrows, args.distance_threshold, args.min_points, args.max_points, args.save
    )
