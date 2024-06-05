#! /usr/bin/env python3

import argparse
import json
import os
import sys
import warnings
from pprint import pprint

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from sklearn.metrics import (
    ConfusionMatrixDisplay,
    balanced_accuracy_score,
    confusion_matrix,
    f1_score,
    precision_score,
    recall_score,
)
from sklearn.preprocessing import label_binarize

warnings.filterwarnings("ignore")
from util import *


def arguments():
    parser = argparse.ArgumentParser(description="Analyze results")
    parser.add_argument(
        "radius",
        type=float,
        nargs="?",
        default=0.0000001,
        help="Radius in kilometers to consider a point as a match (default: 0.1)",
    )
    parser.add_argument(
        "-d",
        "--distance-matrix",
        type=str,
        help="Path to the distance matrix (required)",
    )
    parser.add_argument(
        "-r",
        "--results",
        type=str,
        help="Path to the results (optional, if not provided, read from stdin)",
    )
    parser.add_argument(
        "--plot",
        action="store_true",
        help="Plot the confusion matrix",
        default=False,
    )
    parser.add_argument(
        "--aggregate",
        action="store_true",
        help="Aggregate the results",
        default=False,
    )
    args = parser.parse_args()

    if args.distance_matrix is None:
        parser.print_help()
        sys.exit(1)

    args.distance_matrix = read_distance_matrix(args.distance_matrix)
    if args.results is not None:
        args.results = read_results(result_file_path(args.results))
    elif not sys.stdin.isatty():
        args.results = results_from_json(json.loads(sys.stdin.read()))
    else:
        parser.print_help()
        sys.exit(1)

    return args


def read_results(path):
    results = pd.read_parquet(path)
    results.set_index("query", inplace=True)
    return results


def predicted_labels(datapoints, results):
    return np.isin(datapoints, results.values.flatten()).astype(int)


def true_labels(matrix, r):
    return (
        label_binarize(matrix.values <= r, classes=matrix.columns.values)
        .any(axis=0)
        .astype(int)
    )


def confusion_matrix_labels(cf_matrix):
    group_counts = cf_matrix.flatten()
    group_percentages = group_counts / np.sum(cf_matrix)
    group_names = ["True Negative", "False Negative", "False Negative", "True Positive"]
    return np.array(
        [
            f"{name}\n{count}\n{percent}"
            for name, count, percent in zip(
                group_names, group_counts, group_percentages
            )
        ]
    ).reshape(2, 2)


def plot_confusion_matrix(y_true, y_pred):
    matrix = confusion_matrix(y_true, y_pred, labels=[True, False])
    disp = ConfusionMatrixDisplay(
        confusion_matrix=matrix,
        display_labels=[True, False],
    )
    disp.text_ = confusion_matrix_labels(matrix)
    disp.plot(cmap=plt.cm.Blues, xticks_rotation="horizontal")
    plt.show()


def scores_single_query(y_true, y_pred, zero_division=np.nan):
    if np.all(y_true == 0) and np.all(y_pred == 0):
        return (0.0, 0.0, 0.0)
    else:
        return (
            precision_score(y_true, y_pred, zero_division=zero_division),
            recall_score(y_true, y_pred, zero_division=zero_division),
            f1_score(y_true, y_pred, zero_division=zero_division),
        )


def create_pred_matrix(distance_matrix, candidates):
    n_res, n_candidates = candidates.shape
    n_queries, n_data = distance_matrix.shape
    pred_matrix = pd.DataFrame(
        data=np.zeros_like(distance_matrix, dtype=bool),
        index=distance_matrix.index,
        columns=distance_matrix.columns,
        dtype=bool,
    )
    for i, v in zip(candidates.index, candidates.values.flatten()):
        pred_matrix.loc[i, v] = True

    return pred_matrix


def scores_queryset(y_true_matrix, y_pred_matrix):
    return np.array(
        [
            scores_single_query(y_true_matrix.loc[i], y_pred_matrix.loc[i])
            for i in y_pred_matrix.index
        ]
    ).reshape(-1, 3)


def create_result_object(scores, truth_matrix, pred_matrix):
    scores = scores.astype(float)
    return {
        "precision": np.nanmean(scores[:, 0]),
        "recall": np.nanmean(scores[:, 1]),
        "f1": np.nanmean(scores[:, 2]),
        "precision_min": scores[:, 0].min(),
        "recall_min": scores[:, 1].min(),
        "f1_min": scores[:, 2].min(),
        "precision_max": scores[:, 0].max(),
        "recall_max": scores[:, 1].max(),
        "f1_max": scores[:, 2].max(),
        "truth": int(truth_matrix.values.sum()),
        "pred": int(pred_matrix.values.sum()),
    }


def analyze_results(y_truth_matrix, results, radius):
    """
    :param y_truth_matrix: DataFrame, binary matrix with the ground truth
    :param results: list[obj] with value of "candidates" as a DataFrame with results candidate pairs
    :param radius: float, radius in geographical coordinate distance

    :return: list[obj] with the following
        {
            "config": obj, configuration of the results
            "scores": {
                "precision": float,
                "recall": float,
                "f1": float,
                "precision_min": float,
                "recall_min": float,
                "f1_min": float,
                "precision_max": float,
                "recall_max": float,
                "f1_max": float,
                "truth": int,
                "pred": int,
            }
        }
    """
    return [analyze_single_result(y_truth_matrix, res) for res in results]


from sklearn.metrics import precision_recall_curve

# Plot the precision-recall curve
# plt.figure()
# plt.plot(recall, precision, marker=".", label="Precision-Recall curve")
# plt.xlabel("Recall")
# plt.ylabel("Precision")
# plt.title("Precision-Recall Curve")
# plt.legend()
# plt.grid(True)
# plt.show()


def analyze_single_result(y_truth_matrix, result):
    query_result = result["candidates"]
    query_index = query_result.index.unique()
    pred_matrix = create_pred_matrix(y_truth_matrix, query_result)
    precision, recall, _ = precision_recall_curve(
        y_truth_matrix.to_numpy().flatten(), pred_matrix.to_numpy().flatten()
    )
    assert y_truth_matrix.shape == pred_matrix.shape, "Matrix shapes do not match"
    TP = int((y_truth_matrix & pred_matrix).sum().sum())
    FP = int((~y_truth_matrix & pred_matrix).sum().sum())
    FN = int((y_truth_matrix & ~pred_matrix).sum().sum())
    TN = int((~y_truth_matrix & ~pred_matrix).sum().sum())
    precision = TP / (TP + FP) if TP + FP != 0 else np.nan
    recall = TP / (TP + FN) if TP + FN != 0 else np.nan
    f1 = (
        2 * precision * recall / (precision + recall)
        if precision + recall != 0
        else np.nan
    )
    return {
        "config": result["config"],
        "TP": int(TP),
        "FP": int(FP),
        "FN": int(FN),
        "TN": int(TN),
        "precision": round(precision, 2),
        "recall": round(recall, 2),
        "f1": round(f1, 2),
    }


def aggregate_results(results):
    precision = [r["precision"] for r in results]
    recall = [r["recall"] for r in results]
    f1 = [r["f1"] for r in results]
    best_i = f1.index(max(f1))
    worst_i = f1.index(min(f1))
    # return the precision, recall and f1 scores for the worst, best and average results
    # as well as the average number of true and predicted matches
    return {
        "precision": {
            "best": float(precision[best_i]),
            "avg": float(round(np.nanmean(precision), 2)),
            "worst": float(precision[worst_i]),
        },
        "recall": {
            "best": float(recall[best_i]),
            "avg": float(round(np.nanmean(recall), 2)),
            "worst": float(recall[worst_i]),
        },
        "f1": {
            "best": float(f1[best_i]),
            "avg": float(round(np.nanmean(f1), 2)),
            "worst": float(f1[worst_i]),
        },
        "TP": {
            "avg": float(np.nanmean([r["TP"] for r in results])),
        },
        "FP": {
            "avg": float(np.nanmean([r["FP"] for r in results])),
        },
        "FN": {
            "avg": float(np.nanmean([r["FN"] for r in results])),
        },
        "TN": {
            "avg": float(np.nanmean([r["TN"] for r in results])),
        },
    }


def create_result_config(result, y_truth_matrix):
    y_pred_matrix = create_pred_matrix(y_truth_matrix, result["candidates"])
    return {
        "config": result["config"],
        "scores": create_result_object(
            scores_queryset(
                y_truth_matrix,
                y_pred_matrix,
            ),
            y_truth_matrix,
            y_pred_matrix,
        ),
    }


def main():
    args = arguments()
    distance_matrix = args.distance_matrix
    truth_matrix = distance_matrix <= args.radius
    if isinstance(args.results, list):
        if args.aggregate:
            print(
                json.dumps(
                    aggregate_results(
                        analyze_results(truth_matrix, args.results, args.radius)
                    ),
                    indent=4,
                )
            )
        else:
            print(
                json.dumps(
                    analyze_results(truth_matrix, args.results, args.radius), indent=4
                )
            )
    else:
        print(json.dumps(analyze_single_result(truth_matrix, args.results), indent=2))


if __name__ == "__main__":
    main()
