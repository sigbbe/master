import math
import os
import sys
from pprint import pprint

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def reachable_vectors(l, r, sigma):
    """
    Calculate the number of reachable vectors at level l
    N(l) = Σ (l choose k) x (σ - 1)^k
    Args:
    - l: Level of the node
    - r: Maximum allowed errors
    - sigma: Alphabet size

    Returns:
    - Number of reachable vectors N(l)
    """
    return sum(math.comb(l, k) * (sigma - 1) ** k for k in range(r + 1))


def reachable_vectors_r(l, r, sigma):
    """
    Calculate the number of reachable vectors at level l with exactly r errors
    N_2(l) = (l choose r) x (σ - 1)^r
    Args:
    - l: Level of the node
    - r: Maximum allowed errors
    - sigma: Alphabet size
    """
    return math.comb(l, r) * (sigma - 1) ** r


def reach_probability(l, r, sigma):
    """
    Calculate reach probability for a node at level l
    P(l) = N(l) / σ^r if l > r else 1
    Args:
    - l: Level of the node
    - r: Maximum allowed errors
    - sigma: Alphabet size

    Returns:
    - Reach probability P(l)
    """
    if l <= r:
        return 1
    else:
        N_l = sum(math.comb(l, k) * (sigma - 1) ** k for k in range(r + 1))
        return N_l / sigma**r


def computational_cost(l, r, sigma):
    """
    Calculate computational cost for a node at level l
    F_in(l) = (1 - (N_2(l) / N(l))) x σ +(N_2(l) / N(l))
    Args:
    - l: Level of the node
    - r: Maximum allowed errors
    - sigma: Alphabet size
    """
    return (
        1 - (reachable_vectors_r(l, r, sigma) / reachable_vectors(l, r, sigma))
    ) * sigma + (reachable_vectors_r(l, r, sigma) / reachable_vectors(l, r, sigma))


def search_cost_inner_node(l, r, sigma):
    """
    Calculate search cost for an inner node v at level l
    C_in(v) = P(l) x F_in(l)
    """
    return reach_probability(l, r, sigma) * computational_cost(l, r, sigma)


def search_cost_leaf_node(l, r, sigma, children):
    """
    Calculate search cost for a leaf node v at level l
    C_leaf(v) = P(l) x log_2(σ) x |L_v|
    """
    return reach_probability(l, r, sigma) * math.ceil(math.log2(sigma)) * children


def split_threshold(l, r, sigma):
    """
    Calculate split threshold τ∗ for a node at level l
    """
    return (
        reach_probability(l, r, sigma)
        / (reach_probability(l, r, sigma) + reach_probability(l + 1, r, sigma))
        * (computational_cost(l, r, sigma) / math.ceil(math.log2(sigma)))
    )


def compute_thresholds(matrix):
    (bits, radius, levels) = matrix.shape
    for bit in range(bits):
        for r in range(radius):
            for l in range(r, levels):
                sigma = int(math.pow(2, bit + 1))
                matrix[bit][r][l] = split_threshold(l, r, sigma=sigma)

    return matrix


def inner_plot_split_threshold(thresholds, bits=8, radius=16, max_level=20, **kwargs):
    min_level = radius
    thresholds = np.concatenate(
        (np.zeros(shape=(radius,)), thresholds[bits - 1][radius][radius:max_level]),
        axis=0,
    )
    levels = np.arange(1, len(thresholds) + 1)
    plt.plot(levels[min_level:], thresholds[min_level:], **kwargs)


def plot_split_threshold(thresholds, bits=8, radius=16, max_level=40, **kwargs):
    plt.title(f"Optimal thresholds τ∗ for σ={SIGMA}")
    for r in range(radius):
        inner_plot_split_threshold(
            thresholds,
            bits=bits,
            radius=r,
            max_level=max_level,
            label=f"r={r+1}",
            **kwargs,
        )
    plt.xlabel("Level")
    plt.ylabel("Split Threshold")
    plt.xticks(np.arange(0, max_level, 1))
    plt.tight_layout()
    plt.legend()
    plt.show()


SIGMA = 2
RADIUS = 4
BITS = int(math.log2(SIGMA))

MAX_SIGMA = 256  # Alphabet size
MAX_BITS = int(math.log2(MAX_SIGMA))
MAX_RADIUS = 17 - 1  # Maximum allowed errors
MAX_LEVELS = 64  # level

PRINT_THRESHOLDS = False


def main():
    computed_thresholds = compute_thresholds(
        np.zeros((MAX_BITS, MAX_RADIUS, MAX_LEVELS))
    )
    computed_thresholds = computed_thresholds * 4
    plot_split_threshold(computed_thresholds, bits=BITS, radius=RADIUS)

    if PRINT_THRESHOLDS:
        pprint(computed_thresholds.tolist())


if __name__ == "__main__":
    main()
