#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import os

import msgpack as mp
import pandas as pd
from util import ID_COLUMN, LAT_COLUMN, LON_COLUMN, dataset_name, result_dir


def msg_pack_buffer(df: pd.DataFrame, index: pd.Index):
    return b"".join(
        [
            mp.packb(
                {
                    "id": idx,
                    "points": [[float(x[0]), float(x[1])] for x in df.loc[idx].values],
                }
            )
            for idx in filter(lambda x: len(df.loc[x].shape) == 2, index)
        ]
    )


def write_msg_pack(path: str, buf: bytes):
    print(path)
    return (f := open(path, "wb")).write(buf) > 0 and f.close()


def result_file(path):
    return os.path.join(result_dir(), f"{dataset_name(path)}.msgpack")


def query_result_file(path):
    return os.path.join(result_dir(), f"{dataset_name(path)}-query.msgpack")


def main(args):
    path = args[1]
    queries = list(map(lambda x: int(x), args[2:])) if len(args) > 2 else None
    df = pd.read_parquet(path)
    data_index = df.index.unique()
    if queries:
        query_index = df.index.intersection(queries)
        data_index = df.index.difference(query_index)
        data = msg_pack_buffer(df, data_index)
        query = msg_pack_buffer(df, query_index)
        code = write_msg_pack(query_result_file(path), query)
        code = write_msg_pack(result_file(path), data) and code
        return code
    else:
        return write_msg_pack(result_file(path), msg_pack_buffer(df, data_index))

    return 1


if __name__ == "__main__":
    import sys

    sys.exit(main(sys.argv))
