#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import json
import os
import random
import zipfile
from enum import Enum
from typing import Optional

import numpy as np
import pandas as pd
import polyline
from geopy.distance import geodesic
from tqdm import tqdm

ID_COLUMN = "id"
LAT_COLUMN = "lat"
LON_COLUMN = "lon"
DATA_DIR = "data"


class IndexType(Enum):
    DYFT = 0
    FRESH = 1
    RESULT_ERROR = 2


def trajectory(df: pd.DataFrame, id: int) -> pd.Series:
    return df[df[ID_COLUMN] == id]


def max_trajectory_length(df: pd.DataFrame) -> int:
    return df.groupby(ID_COLUMN).size().max()


def map_center(df):
    return np.array([df[LAT_COLUMN].mean(), df[LON_COLUMN].mean()])


def filter_distance_inner(df: pd.DataFrame, row: pd.Series, length: int):
    if row.name == 0 or row.name == length - 1:
        return True  # Always include the first and last points

    [prev_lat, prev_lon] = df.iloc[row.name - 1][[LAT_COLUMN, LON_COLUMN]]
    [next_lat, next_lon] = df.iloc[row.name + 1][[LAT_COLUMN, LON_COLUMN]]
    [lat, lon] = row[[LAT_COLUMN, LON_COLUMN]]

    distance_prev = geodesic(
        (lat, lon),
        (prev_lat, prev_lon),
    )

    distance_next = geodesic(
        (lat, lon),
        (next_lat, next_lon),
    )

    return distance_prev.kilometers <= 1 and distance_next.kilometers <= 1


""" def filter_distance(df: pd.DataFrame) -> pd.DataFrame:
# filter out points that are more than 1 km away from the previous and next pointss
return df[df.apply(lambda x: filter_distance_inner(df, x, len(df)), axis=1)] """


def filter_distance(
    df: pd.DataFrame, km: float, min_points: int, max_points: int
) -> pd.DataFrame:
    # Group by trajectory ID and iterate over each trajectory
    grouped = df.groupby(ID_COLUMN)
    filtered_trajectories = []

    for tid, trajectory in grouped:
        # Calculate distances between consecutive points
        lat_diff = trajectory[LAT_COLUMN].diff()
        lon_diff = trajectory[LON_COLUMN].diff()
        distance = (
            np.sqrt(lat_diff**2 + lon_diff**2) * 111.32
        )  # Approximate conversion from degrees to kilometers

        # Check if any distance exceeds the specified threshold
        if (distance.shift(-1) >= km).any() or (distance.shift(1) >= km).any():
            # print(f"Trajectory {tid} exceeds the distance threshold of {km} km")
            continue  # Skip this trajectory if any point exceeds the distance criterion

        # Check if the number of points in the trajectory is below the threshold
        if len(trajectory) < min_points:
            # print(f"Trajectory {tid} has fewer than {min_points} points")
            continue  # Skip this trajectory if it has fewer points than the specified threshold

        # Check if the number of points in the trajectory is below the threshold
        if len(trajectory) > max_points:
            # print(f"Trajectory {tid} has more than {max_points} points")
            continue  # Skip this trajectory if it has fewer points than the specified threshold

        filtered_trajectories.append(trajectory)

    # Concatenate filtered trajectories into a single DataFrame
    if len(filtered_trajectories) > 0:
        return pd.concat(filtered_trajectories)
    return None


def polyline_str(traj, p=5):
    return polyline.encode(
        traj[[LAT_COLUMN, LON_COLUMN]].values.tolist(), precision=p, geojson=False
    )


def dataset_name(path):
    return os.path.basename(path).split(".")[0]


def result_dir():
    return os.path.abspath(os.environ.get("MASTER_RESULT_DIR", "results"))


def query_dir():
    return os.path.abspath(os.environ.get("MASTER_QUERY_DIR", "queries"))


def data_dir():
    return os.path.abspath(os.environ.get("MASTER_DATA_DIR"))


def map_file_path(file, base_dir=None):
    if base_dir is not None:
        if isinstance(file, str):
            return os.path.join(base_dir, file)
        if isinstance(file, list):
            return [os.path.join(base_dir, f) for f in file]
    else:
        return file


def results_from_json(data):
    def prepare_single_result(res):
        metadata, candidates = {
            key: value for key, value in res.items() if key != "candidates"
        }, res["candidates"]
        candidates = pd.DataFrame(
            {
                np.int64(key): np.int64(value) for key, value in candidates.items()
            }.items(),
            columns=["query", "candidate"],
        )
        candidates.set_index("query", inplace=True)
        candidates = candidates.explode("candidate")
        candidates = candidates.astype(np.int64)
        metadata["candidates"] = candidates
        return metadata

    if isinstance(data, list):
        return [prepare_single_result(d) for d in data]
    else:
        return prepare_single_result(data)


def read_distance_matrix(path):
    distance_matrix = pd.read_parquet(
        path,
    )
    distance_matrix.index = distance_matrix.index.astype(int)
    distance_matrix.set_index(ID_COLUMN, inplace=True)
    distance_matrix.columns = distance_matrix.columns.astype(int)
    return distance_matrix.T


def dataset_file_path(file):
    return os.path.abspath(map_file_path(file, data_dir()))


def query_file_path(file):
    return map_file_path(file, query_dir())


def result_file_path(file):
    return map_file_path(file, result_dir())


def in_file(file):
    return os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", DATA_DIR, "raw", file)
    )


def out_file(file):
    return os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", DATA_DIR, "processed", file)
    )


def preprocess_args():
    import argparse

    args = argparse.ArgumentParser(description="Preprocess trajectory data")
    n = args.add_argument_group("Number of rows")
    n.add_argument(
        "-n",
        "--nrows",
        type=int,
        default=1000,
        help="Number of rows to read from the input file (default: 1000)",
    )
    n.add_argument("-a", "--all", action="store_true", help="Read all rows")
    args.add_argument(
        "-min-p",
        "--min-points",
        type=int,
        default=5,
        help="Minimum number of points in a trajectory (default: 5)",
    )
    args.add_argument(
        "-max-p",
        "--max-points",
        type=int,
        default=1000,
        help="Maximum number of points in a trajectory (default: 1000)",
    )
    args.add_argument(
        "-dt",
        "--distance-threshold",
        type=float,
        default=1.0,
        help="Maximum distance between consecutive points in kilometers (default: 1.0 km)",
    )

    args.add_argument(
        "--save",
        action="store_true",
        default=False,
        help="Save the preprocessed data to a file (default: False)",
    )

    return args.parse_args()
