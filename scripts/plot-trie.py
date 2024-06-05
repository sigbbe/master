#! /usr/bin/env python3
# -*- coding: utf-8 -*-

import collections
import os

import graphviz as gv
import matplotlib.pyplot as plt
import msgpack as mp
import numpy as np

def flatten(nested):
    queue = collections.deque(nested)
    out = []
    while queue:
        e = queue.popleft()
        if isinstance(e, list):
            queue.extendleft(reversed(e))
        else:
            out.append(e)
    return out


def read_file(path):
    with open(path, "rb") as file:
        return mp.unpackb(file.read())


def create_node(node: [int, [int, str]]) -> dict:
    return {"label": node[0], "id": node[1][0], "type": node[1][1]}


def node_label(node_type: str, node_id: str, children=None) -> str:
    return f"{node_type}[{node_id}]" + (str(hash(str(children))) if children else "")


# Function to recursively add nodes to the graph
def add_nodes(graph, data, parent_node=None):
    if not data:
        return

    if isinstance(data, list):
        for node_data in data:
            match len(node_data):
                case 2:
                    label, [node_id, node_type, children] = node_data
                    node = node_label(node_type, node_id)
                    graph.node(
                        node,
                    )
                    if parent_node:
                        graph.edge(parent_node, node, label=str(label))
                    add_nodes(graph, children, parent_node=node)
                case 3:
                    node_id, node_type, children = node_data
                    node = node_label(node_type, node_id)
                    graph.node(node)
                    if parent_node:
                        graph.edge(parent_node, node)
                    add_nodes(graph, children, parent_node=node)
                case _:
                    print("Node data:", len(node_data))
                    return


def main(args):
    # Read input data
    path = args[1]
    data = read_file(path)
    # Create a Graphviz graph
    dot = gv.Digraph(comment="MART Trie Index")

    # root, data = data[0][:2], data[0][2:]
    # Add nodes to the graph
    add_nodes(dot, data, parent_node="root")

    # Render the graph
    dot.render(
        filename=os.path.splitext(path)[0],
        view=False,
    )


    return 0


if __name__ == "__main__":
    import sys

    sys.exit(main(sys.argv))
