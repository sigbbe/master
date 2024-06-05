# Datasets

- [Analysis of temporal patterns in animal movement networks](https://zenodo.org/records/4932137)

- [Backward Trajectories](https://b2find.dkrz.de/dataset/022d0dd7-2718-5653-aa92-f77cad0c016d)

- [Escape path trajectory data](https://figshare.com/articles/dataset/Escape_path_trajectory_data/4903229)

- [CitySim Dataset](https://paperswithcode.com/dataset/citysim): "This paper introduces the CitySim Dataset, which was devised with a core objective of facilitating safety-based research and applications. CitySim has vehicle trajectories extracted from 1140-minutes of drone videos recorded at 12 different locations. It covers a variety of road geometries including freeway basic segments, weaving segments, expressway merge/diverge segments, signalized intersections, stop-controlled intersections, and intersections without sign/signal control."

- [Shortest Paths in San Francisco](https://sfsp.mpi-inf.mpg.de/)

## File structure

```
data
├── query
│   ├── porto-2-p-10-20.parquet
│   ├── ...
│   └── backward-2-t-0-100000.parquet
├── processed
│   ├── backwards.parquet
│   └── porto.parquet
└── raw
    ├── animal-movement-networks
    ├── backward-trajectories.zip
    ├── escape-path-trajectories.zip
    ├── porto.zip
    └── rome.tar.gz
```

The `data/` directory contains three folder with a defined purpose. After running the download script, the `raw/` directory contains the raw datasets.
The `query/` directory stores the preprocessed datasets. The format of the preprocessed trajectory datasets can be seen below. The trajectory data is stored in a parquet row format, where the there is a single-field index with the numeric trajectory id that is associated with all points on a given trajectory. The id field is a signed 64-bit integer, and the lat- and lon columns are 64-bit floating point numbers.

```txt
                           lat       lon
id
1372636858620000589  41.141412 -8.618643
1372636858620000589  41.141376 -8.618499
1372636858620000589  41.142510 -8.620326
1372636858620000589  41.143815 -8.622153
1372636858620000589  41.144373 -8.623953
...                        ...       ...
1372636858620000589  41.154516 -8.630829
1372636858620000589  41.154498 -8.630829
1372636858620000589  41.154489 -8.630838
1372636858620000589  41.141412 -8.618643
1372636858620000589  41.141376 -8.618499
```

### Porto

We sampled 100 thousand trajectories from the raw dataset by filtering out based on the afformentioned requirements.
Describe the sampled dataset.

- Avg number of points
- Avg length
- hash distribution

Given the variation of curves found in the processed porto dataset, we chose to define a set of quantitative trajectory features to elude performance differences from DyFT and FRESH.

We sampled querysets consisting of 2, 4, 10, 20 trajectories with varying requirements for number of points, the total length, and complexity (i.e., a combination of number of points and number of turns in the trajectory).

| **Requirement**         | **Levels**                                                                                                                                                                             | **Example** |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| Number of sample points | Here, we chose to differentiate between small curves, medium, and large trajectories. We define samll trajectories as having number of points $p \leq 10$ Small, medium as $p \leq 50$ |             |
| Length                  |                                                                                                                                                                                        |             |
| Number of turns         | We define a turn as a point $p$ in the trajectory where the angle between $p$'s previous point, $p$ itself and $p$'s next point exceeds 30 degrees.                                    |             |
|                         |                                                                                                                                                                                        |             |
