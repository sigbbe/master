# Dataset utilities

## Setup

```bash
$ python --version # Python 3.12.2
```

```bash
$ pip --version # pip 23.3.1
```

```bash
$ pip install -r requirements.txt
```

## Method

We have scripts for preprocessing each dataset:

```bash
$ ./py-utils/<dataset-name>.py
```

The scripts for preprocessing the datasets accept the following arguments: `-min-p MIN_POINTS` for filtering out trajectories with less than `MIN_POINTS` sample points, `-max-p MAX_POINTS` for filtering out trajectories with more than `MAX_POINTS`, `-dt DISTANCE_THRESHOLD` for filtering out trajectories containing sample points where the distance between two consecutive points have a distance more than `DISTANCE_THRESHOLD` (i.e., noicy datapoints). The scripts also accept the arguments `-n NROWS` where `NROWS` is the number of trajectories to include, and do not save the preprocessed data unless the `--save` flag is passed.

For example, to prepare the porto dataset requiring each trajectory to have at least 10 points, and no consecutive points being more than 0.5 kilometers apart, we run:

```bash
$ ./scripts/porto.py -min-p 10 -max-p 1000 -dt 1.0 -n 50000 --save
```

extract queryset of 1000 trajectories

```bash
$ ./scripts/extract-trajectories.py porto.parquet --query -n 1000 --seed 42 -o porto-query.parquet
```

extract dataset of 5000 trajectories

```bash
$ ./scripts/extract-trajectories.py porto.parquet -n 5000 --seed 42 -o porto-data.parquet
```

generate distance matrix for the queryset

```bash
$ ./target/release/examples/create-benchmark porto-data.parquet porto-query.parquet distance-matrices/porto-query-matrix.parquet
```

generate distance matrix for the self similarity join

```bash
$ ./target/release/examples/create-benchmark ../query/porto-query.parquet porto-query.parquet distance-matrices/porto-query-self-join-matrix.parquet
```

get percentiles of distances

```bash
$ python scripts/percentile-distances.py results/distance-matrices/porto-query-matrix.parquet 0.01 0.05 0.1 1 5 10
```

run with 20 different seeds to evaluate accuracy

```bash
$ python ./scripts/fuzz-params.py --dataset porto-data.parquet --queryset porto-query.parquet --delta <lsh-grid-delta> dyft | ./scripts/analyze-results.py <frechet-distance> -d results/distance-matrices/porto-query-matrix.parquet
```
