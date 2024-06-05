import os
import sys

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import scipy.spatial.distance as ssd
import seaborn as sns
import similaritymeasures as sm
from Levenshtein import distance as levenshtein_distance
from scipy.stats import pearsonr
from sklearn.cluster import KMeans
from sklearn.decomposition import PCA
from sklearn.preprocessing import StandardScaler

import util


def create_matrix(y_truth_matrix, d_hahes, q_hashes, metric):
    return pd.DataFrame(
        ssd.cdist(
            q_hashes,
            d_hahes,
            metric=metric,
        ),
        index=y_truth_matrix.index,
        columns=y_truth_matrix.columns,
    )


def create_ham_distance_matrix(y_truth_matrix, d_hashes, q_hashes):
    return create_matrix(
        y_truth_matrix, d_hashes, q_hashes, lambda x, y: np.count_nonzero(x != y)
    )


def create_euclidean_distance_matrix(y_truth_matrix, d_hashes, q_hashes):
    return create_matrix(
        y_truth_matrix, d_hashes, q_hashes, lambda x, y: np.linalg.norm(x - y)
    )


def create_edit_distance_matrix(y_truth_matrix, d_hashes, q_hashes):
    return create_matrix(
        y_truth_matrix, d_hashes, q_hashes, lambda x, y: levenshtein_distance(x, y)
    )


def draw_similarity_correlation(
    hash_similarities: pd.DataFrame,
    true_similarities: pd.DataFrame,
    bins: int,
) -> None:
    """
    Method that draws a similarity correlation graph visualising the correlation between
    the true similarities and the hashed similarities.

    :param hash_similarities: The hashed similarities, n by m matrix where n is the number of queries and m is the number of data points
    :param true_similarities: The true similarities, n by m matrix
    :param bins: The number of bins for the histogram
    """
    hash_sim = hash_similarities.to_numpy().flatten()
    true_sim = true_similarities.to_numpy().flatten()

    fig, ax = plt.subplots(dpi=300)
    h = ax.hist2d(
        hash_sim,
        true_sim,
        bins=bins,
        cmap="turbo",
    )
    ax.set_xticks(np.arange(0, np.max(hash_sim), 4))
    ax.set_ylim(0, np.max(true_sim) / 6)
    ax.set_xlim(0, np.max(hash_sim))
    ax.set_ylabel(f"True Fréchet distance (Trajectory)")
    ax.set_xlabel(f"Hamming distance (Hash)")
    ax.tick_params(axis="both", which="major")
    fig.colorbar(h[3], ax=ax)
    plt.show()


def plot_histogram(distance_matrix: pd.DataFrame, bins: int = 100) -> None:
    import matplotlib.pyplot as plt

    # Flatten the distance matrix into a 1D array
    distances = distance_matrix.values.flatten()

    # Plot the histogram
    plt.hist(distances, bins=bins)
    plt.xlabel("Distance")
    plt.ylabel("Frequency")
    plt.title("Histogram of Distance Matrix")
    plt.show()


def main(argv):
    q_hash_path = util.result_file_path(argv[0])
    d_hash_path = util.result_file_path(argv[1])
    distance_matrix_path = util.result_file_path(argv[2])
    q_hashes = pd.read_parquet(q_hash_path).T
    d_hashes = pd.read_parquet(d_hash_path).T
    distance_matrix = util.read_distance_matrix(distance_matrix_path)
    hash_matrix = create_ham_distance_matrix(distance_matrix, d_hashes, q_hashes)
    correlation_matrix = np.corrcoef(
        distance_matrix.values.flatten(), hash_matrix.values.flatten()
    )[0][1]
    print(
        pearsonr(
            hash_matrix.values.flatten(),
            distance_matrix.values.flatten(),
            alternative="greater",
        )[0]
    )
    draw_similarity_correlation(hash_matrix, distance_matrix, 100)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
