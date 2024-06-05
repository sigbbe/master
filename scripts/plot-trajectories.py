#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import json
import os
import subprocess
import sys
import warnings
import zipfile
from typing import Optional

import folium
import numpy as np
import pandas as pd
from geopy.distance import geodesic
from tqdm import tqdm

from util import (
    ID_COLUMN,
    LAT_COLUMN,
    LON_COLUMN,
    data_dir,
    dataset_file_path,
    map_center,
    map_file_path,
    query_file_path,
    result_dir,
    results_from_json,
)

warnings.filterwarnings("ignore")


def trajectory_plot_args():
    parser = argparse.ArgumentParser(description="CLI for plotting trajectories!")
    parser.add_argument("-d", "--dataset", type=str, help="Path to the dataset")
    parser.add_argument(
        "-q", "--queryset", type=str, nargs="+", help="Path to the queryset(s)"
    )
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--ids", nargs="+", type=int, help="List of integer IDs")
    rows = selection.add_mutually_exclusive_group()
    rows.add_argument(
        "-a",
        "--all",
        action="store_true",
        help="Plot all trajectories in the dataset",
    )
    rows.add_argument(
        "-n",
        "--nrows",
        nargs="?",
        type=int,
        help="Number of rows to read from the dataset",
    )
    selection.add_argument(
        "--from-results", nargs="?", type=str, help="Path to results file"
    )
    parser.add_argument("-o", "--output-file", help="Output file")
    return parser.parse_args()


def new_map(df):
    return folium.Map(
        location=map_center(df),
        zoom_start=10,
        control_scale=True,
        control_zoom=True,
        png_enabled=True,
        zoom_control=True,
    )


def parse_args():
    cargs = trajectory_plot_args()
    df = trajectories = pd.read_parquet(dataset_file_path(cargs.dataset))
    if cargs.queryset:
        df = pd.concat(
            [df]
            + [
                pd.read_parquet(query_file_path(file_path))
                for file_path in cargs.queryset
            ]
        )
    args = dict()
    args["dataset"] = df
    if cargs.from_results is not None:
        results = pd.read_parquet(map_file_path(cargs.from_results, result_dir()))
        results.set_index("query", inplace=True)
        args["results"] = results
    elif cargs.nrows:
        args["index"] = df.index.unique()[: cargs.nrows]
    elif cargs.ids:
        args["index"] = pd.Index(cargs.ids)
    else:
        args["index"] = df.index.unique()

    # Create a map centered at the mean latitude and longitude of trajectories
    args["fmap"] = new_map(df)
    args["output_file"] = map_file_path(cargs.output_file, result_dir())
    return args


def plot_trajectories(
    fmap: folium.Map, dataset: pd.DataFrame, index: pd.Index, **kwargs
):
    print(f"Plotting {len(index)} trajectories")
    for tid in index:
        plot_single_trajectory(fmap, dataset, tid, **kwargs)


def plot_single_trajectory(
    fmap: folium.Map, dataset: pd.DataFrame, index: int, **kwargs
):
    polyline = folium.PolyLine(
        dataset.loc[index][[LAT_COLUMN, LON_COLUMN]].values.tolist(),
        popup=folium.Popup(f"ID={index}", sticky=True, lazy=True),
        **kwargs,
    )
    polyline.add_to(fmap)


def plot_trajectories_from_results(
    dataset: pd.DataFrame, results: pd.DataFrame, fmap: folium.Map, **kwargs
):
    print(f"Plotting {len(results)} trajectories")
    # Plot trajectories
    for i, (query_id, candidates) in enumerate(results.groupby("query")):
        idx = pd.Index(candidates["candidate"])
        plot_single_trajectory(
            fmap, dataset, query_id, color="red", weight=8, opacity=0.8, radius=10
        )
        plot_trajectories(fmap, dataset, idx, color=f"#{i+1:02x}{i+1:02x}ff")


if __name__ == "__main__":
    args = None
    if not sys.stdin.isatty():
        args = results_from_json(json.loads(sys.stdin.read()))
        args["dataset"] = pd.read_parquet(dataset_file_path(args["dataset"]))
        if "queryset" in args:
            args["dataset"] = pd.concat(
                [
                    args["dataset"],
                    pd.read_parquet(query_file_path(args["queryset"])),
                ]
            )
        args["fmap"] = new_map(args["dataset"])
        args["results"] = args["candidates"]
        args["output_file"] = os.path.splitext(args["queryset"])[0] + ".html"
        del args["candidates"]
    else:
        args = parse_args()

    if "results" in args:
        plot_trajectories_from_results(**args)
    else:
        plot_trajectories(**args)

    if args["output_file"]:
        args["fmap"].save(args["output_file"])
        sys.exit(
            subprocess.run(
                ["open", args["output_file"]], check=False, capture_output=False
            ).returncode
        )
