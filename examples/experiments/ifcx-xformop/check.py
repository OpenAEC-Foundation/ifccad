"""Manual example comparison; not a CAD converter or a certified accuracy test."""

import json
import math
from fractions import Fraction
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def load(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def exact(values):
    return [Fraction(value) for value in values]


def cross(x, y):
    return [x[1] * y[2] - x[2] * y[1],
            x[2] * y[0] - x[0] * y[2],
            x[0] * y[1] - x[1] * y[0]]


def place(point, frame):
    origin, x, y = (exact(frame[key]) for key in ("origin", "xAxis", "yAxis"))
    z = cross(x, y)
    return [origin[i] + point[0] * x[i] + point[1] * y[i] + point[2] * z[i]
            for i in range(3)]


def native_points(path, nodes):
    attrs = nodes[path]["attributes"]
    if "ifccad::geom::planarPolyline" in attrs:
        geometry = attrs["ifccad::geom::planarPolyline"]
        if geometry["closed"] or any(geometry.get("bulges", [])):
            raise ValueError("This probe only evaluates open straight paths")
        points = [exact(vertex + [0]) for vertex in geometry["vertices"]]
        if "usd::xformop" in attrs:
            return apply_matrix(points, attrs["usd::xformop"]["transform"])
        return [place(point, attrs["ifccad::geom::placement"]) for point in points]
    if "ifccad::geom::lineSegment" in attrs:
        return [exact(attrs["ifccad::geom::lineSegment"][key])
                for key in ("start", "end")]
    instance = attrs["ifccad::blockInstance"]
    transform = instance["transform"]
    if transform["rotation"] != 0:
        raise ValueError("This probe does not implement trigonometric evaluation")
    definition = nodes[instance["definition"]]
    base = exact(definition["attributes"]["ifccad::blockDefinition"]["basePoint"])
    scale = exact(transform["scale"])
    points = []
    for _, child in sorted(definition["children"].items(), key=lambda pair: int(pair[0])):
        for point in native_points(child, nodes):
            local = [(point[i] - base[i]) * scale[i] for i in range(3)]
            points.append(place(local, transform["placement"]))
    return points


def apply_matrix(points, matrix):
    if len(matrix) != 4 or any(len(row) != 4 for row in matrix):
        raise ValueError("Expected a 4 by 4 matrix")
    if any(not math.isfinite(value) for row in matrix for value in row):
        raise ValueError("Nonfinite matrix component")
    if [row[3] for row in matrix] != [0, 0, 0, 1]:
        raise ValueError("Expected an affine row-vector matrix")
    matrix = [exact(row) for row in matrix]
    return [[sum((point + [Fraction(1)])[i] * matrix[i][j]
                 for i in range(4)) for j in range(3)] for point in points]


def main():
    native = {node["path"]: node for node in load("native-placement.ifcx")["data"]}
    standard = {node["path"]: node for node in load("standard-xformop.ifcx")["data"]}
    results = []
    for case in load("cases.json")["cases"]:
        source = native_points(f'/cad/d1/e{case["nativeEntity"]}', native)
        target = native_points(case["standardNode"], standard)
        if source != target:
            raise AssertionError(f'{case["name"]}: native and standard coordinates differ')
        if len(source) != len(case["expectedPoints"]):
            raise AssertionError("Point count differs")
        for point, expected in zip(source, case["expectedPoints"]):
            # Expected decimals are readable summaries, not the exact binary64 inputs.
            if any(abs(float(value) - reference) > 1e-12
                   for value, reference in zip(point, expected)):
                raise AssertionError(f'{case["name"]}: differs from independent expected points')
        results.append({"case": case["name"], "exactStoredScalarEquivalence": True,
                        "points": [[float(value) for value in point] for point in target]})
    base, origin = float(2**54), 1.0
    separate = origin + (base - base)
    combined = base + (origin - base)
    if separate != 1 or combined != 0:
        raise AssertionError("Base-point cancellation witness changed")
    print(json.dumps({"cases": results,
                      "binary64BasePointWitness": {"separate": separate, "combined": combined},
                      "cadFileExchange": "run separately with Rust xformop_exchange tests"}, indent=2))


if __name__ == "__main__":
    main()
