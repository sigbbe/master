# R-Range Search: Comparison of Filter-Tries and Hash Tables for Trajectory Similarity

DyFT (Dynamic Filter Trie) is a scalable indexing method that leverages LSH for similarity search. DyFT is designed to work efficiently over binary and integer vectors and supports dynamic updates for inserting and deleting data points, adapting well to dynamic settings. The main idea of DyFT is to filter similar data points by solving the general hamming distance problem - traversing the trie while allowing candidate vectors to differ (i.e., similarity search with a parameterized radius).

Ceccarello et al.’s Fréchet Similarity with Hashing (FRESH) is an approximate and randomized approach for r-range search under the continuous Fréchet distance. FRESH comprises a filtering and a refinement component. The filtering is based on Driemel and Silvestri’s LSH scheme; candidate near neighbors are selected based on collisions with the query curve. The refinement step reduces false positives by verifying the continuous Fréchet distance. The algorithm provides a tradeoff between performance and quality by parameterizing the numberof hash functions (L) and a refinement threshold (τ). According to their experimentalevaluations, FRESH demonstrates effectiveness in speedup compared to exact solutions, especially when balancing recall and precision

### Parameters

#### LSH

**length**

The number of integer hash values for a single trajectory hash.

**concatenations**

The number of hash values to concatenate to produce a single hash values.

**resolution**

The grid resolution is defined as the absolute distance between two consecutive grid points. The value is treated as is, the distance on a 2 dimensional plane.

#### DyFT

**bits**

The **bits** parameter specifies the number of bits to use for each hash of a trajectory signature. The range of valid values for the bits parameters is 1-8. With bits=1 the trajectory hashes are binary vectors, and with bits=8 each value is one byte.

**errors**

The number of errors allowed when searching the trie structure for candidate hashes. The range of valid values for the errors parameter is 1-16.

**radius**

The hamming distance threshold for candidates reached by searching the trie structure within **errors** errors of the query trajectory.

**splitthreshold**

Splitthreshold defines the upper threshold for number of values associated with a leaf. When inserting a vector if the number of leaves exceeds the value $splitthreshold \cdot in\_weight$, the parent node of the reached leaf node gets split into a smaller node, and values previously associated with the node get moved. If **splitthreshold** is not defined, DyFT uses the precomputed split thresholds that are based on the reach probability of a node at level $l$ within radius $errors$.

**in_weight**

The weighting factor for the **splitthreshold** parameter. If not specified, it's set to 1.0.

#### FRESH

**verify_fraction**

The fraction of data points that collide with the query trajectory (non-zero score) that are to be verified. FRESH verifies only the lowest scored candidates. However, with verify_fraction=1 all found candidates are verified.

### Setup

_Install rustup_

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

_Use nightly version of rustc_

```bash
$ rustup toolchain install nightly
```

_Build the project_

```bash
$ cargo build --release
```

### Environemt

The application respects the following environment variables:

| **Variable**        | **Usage**                                             |
| ------------------- | ----------------------------------------------------- |
| `MASTER_DATA_DIR`   | if set, used as prefix path for `--dataset` arguments |
| `MASTER_QUERY_DIR`  | if set, used as prefix path for `--queryset`          |
| `MASTER_CONFIG_DIR` | if set, used ass prefix path for `--config`           |
| `MASTER_RESULT_DIR` | directory for results output                          |

### Setting up the paper experiments

1. Download the datasets
2. Preprocess the trajectories
3. Extract querysets
4. Precompute distance matrices for querysets

### Running the application

```bash
$ cargo b -r
```

Run either:

```bash
$ ./target/release/dyft -d <dataset>.parquet -q <queryset>.parquet -c <config>.toml -o <output>.parquet
```

or

```bash
$ ./target/release/fresh -d <dataset>.parquet -q <queryset>.parquet -c <config>.toml -o <output>.parquet
```

View the results

```bash
$ python ./py-utils/plot-trajectories.py -d <dataset>.parquet -q <queryset>.parquet --from-results <output>.parquet -o <output-visualization>.html
```

## References

```bib
@inproceedings{kanda2020dynamic,
  author = {Kanda, Shunsuke and Tabei, Yasuo},
  title = {Dynamic Similarity Search on Integer Sketches},
  booktitle = {Proceedings of the 20th IEEE International Conference on Data Mining (ICDM)},
  pages={242-251},
  year = {2020}
}

@inproceedings{ceccarello_fresh_2019,
	author = {Ceccarello, Matteo and Driemel, Anne and Silvestri, Francesco},
	title = {{FRESH}: {Fréchet} {Similarity} with {Hashing}},
	booktitle = {Algorithms and {Data} {Structures}},
	pages = {254-268},
	year = {2019},
	publisher = {Springer International Publishing},
	editor = {Friggstad, Zachary and Sack, Jörg-Rüdiger and Salavatipour, Mohammad R},
	keywords = {Locality Sensitive Hashing, Fréchet distance, Algorithm engineering, Range reporting, Similarity search},
}

@article{driemel_locality-sensitive_2017,
	title = {Locality-sensitive hashing of curves},
	author = {Driemel, Anne and Silvestri, Francesco},
	year = {2017},
	keywords = {Computer Science - Computational Geometry, Computer Science - Data Structures and Algorithms, Computer Science - Information Retrieval, F.2.2},
	pages = {16 pages},
  annote = {Comment: Proc. of 33rd International Symposium on Computational Geometry (SoCG), 2017},
}

```
