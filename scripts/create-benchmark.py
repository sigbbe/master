#! /usr/bin/env python3
# -*- coding: utf-8 -*-
import sys
import threading
from multiprocessing import Pool

import numpy as np
import pandas as pd
import similaritymeasures as sm
from tqdm import tqdm

from util import (
    ID_COLUMN,
    LAT_COLUMN,
    LON_COLUMN,
    dataset_file_path,
    query_file_path,
    result_file_path,
)


def create_benchmark_args():
    import argparse

    args = argparse.ArgumentParser(
        description="Calculate distances between trajectories"
    )
    args.add_argument(
        "dataset",
        type=str,
        help="Path to the input file containing the trajectory data",
    )
    args.add_argument(
        "queryset",
        type=str,
        help="Path to the input file containing the query trajectory data",
    )
    args.add_argument(
        "--output",
        type=str,
        default=None,
        help="Save the evaluation result to a CSV file (default: False)",
    )
    args.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        default=False,
        help="Print verbose output (default: False)",
    )
    return args.parse_args()


"""
@article{Jekel2019,
author = {Jekel, Charles F and Venter, Gerhard and Venter, Martin P and Stander, Nielen and Haftka, Raphael T},
doi = {10.1007/s12289-018-1421-8},
issn = {1960-6214},
journal = {International Journal of Material Forming},
month = {may},
title = {{Similarity measures for identifying material parameters from hysteresis loops using inverse analysis}},
url = {https://doi.org/10.1007/s12289-018-1421-8},
year = {2019}
}
"""


def evaluate_trajectory(data, query, qid, df):
    sim = np.array(
        [
            sm.frechet_dist(
                query, data.loc[dataset_id, [LAT_COLUMN, LON_COLUMN]].values
            )
            for dataset_id in data.index.unique()
        ]
    )
    df[qid] = sim


def create_benchmark(data_df, query_df):
    """
    Function to evaluate dataset trajectories against a set of IDs.
    """
    qidx = query_df.index.unique()
    df = pd.DataFrame(index=data_df.index.unique(), columns=qidx)
    pool = []
    for id_to_evaluate in tqdm(qidx):
        # create a new thread for each query trajectory
        # For each trajectory to evaluate, compare with all trajectories in the dataset
        # Calculate DFD between trajectory to evaluate and all trajectories in the dataset
        thread = threading.Thread(
            target=evaluate_trajectory,
            args=(
                data_df,
                query_df.loc[id_to_evaluate][[LAT_COLUMN, LON_COLUMN]].values,
                id_to_evaluate,
                df,
            ),
        )
        thread.start()
        pool.append(thread)

    [thread.join() for thread in pool]

    return df


def main(dataset, queryset, output, verbose):
    # Load the dataset
    data_df = pd.read_parquet(dataset_file_path(dataset))
    query_df = pd.read_parquet(query_file_path(queryset))

    # Evaluate dataset trajectories
    evaluation_result = create_benchmark(data_df, query_df)

    if verbose:
        print(evaluation_result)

    if output:
        # Save the evaluation result to a CSV file
        evaluation_result.to_parquet(result_file_path(output), index=True)

    return 0


if __name__ == "__main__":
    args = create_benchmark_args()
    sys.exit(main(args.dataset, args.queryset, args.output, args.verbose))
