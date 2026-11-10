"""Dependency-free structural lint for the spike's STEP files (ISO 10303-21).

Usage: python p21_lint.py FILE.step [FILE.step ...]

Checks: header/schema, every #ref resolves, every EDGE_CURVE is used by exactly two
ORIENTED_EDGEs with opposite orientation flags, and prints solid/product names.
Independent of truck's own reader; it does not check geometry.
"""

import re
import sys
from collections import defaultdict


def lint(path: str) -> bool:
    text = open(path, encoding="ascii").read()
    ok = True
    assert text.startswith("ISO-10303-21;"), "missing ISO-10303-21 magic"
    assert text.rstrip().endswith("END-ISO-10303-21;"), "missing END-ISO-10303-21"
    schema = re.search(r"FILE_SCHEMA\(\((.*?)\)\);", text).group(1)
    data = text.split("DATA;", 1)[1].rsplit("ENDSEC;", 1)[0]
    # Entities may span lines (complex instances); split on ';' at the end of an instance.
    ents = {}
    for m in re.finditer(r"#(\d+)\s*=\s*(.*?);\s*(?=#\d+\s*=|$)", data, re.S):
        ents[int(m.group(1))] = " ".join(m.group(2).split())
    refs = {int(r) for body in ents.values() for r in re.findall(r"#(\d+)", body)}
    dangling = sorted(refs - ents.keys())
    if dangling:
        ok = False
        print(f"  DANGLING refs: {dangling[:10]}")
    # Effective direction of an edge in a face = ORIENTED_EDGE flag, flipped when the
    # FACE_BOUND flag is .F. (truck writes .F. bounds on reversed faces).
    loop_flag = {}
    for body in ents.values():
        m = re.match(r"FACE_BOUND\('[^']*', #(\d+), \.(T|F)\.\)", body)
        if m:
            loop_flag[int(m.group(1))] = m.group(2) == "T"
    uses = defaultdict(list)
    for i, body in ents.items():
        m = re.match(r"EDGE_LOOP\('[^']*', \((.*)\)\)", body)
        if not m:
            continue
        for oe in re.findall(r"#(\d+)", m.group(1)):
            om = re.match(r"ORIENTED_EDGE\('[^']*', \*, \*, #(\d+), \.(T|F)\.\)", ents[int(oe)])
            fwd = (om.group(2) == "T") == loop_flag[i]
            uses[int(om.group(1))].append("T" if fwd else "F")
    edges = [i for i, b in ents.items() if b.startswith("EDGE_CURVE(")]
    bad = [e for e in edges if sorted(uses[e]) != ["F", "T"]]
    if bad:
        ok = False
        print(f"  edges not used exactly once each way: {bad[:10]}")
    types = defaultdict(int)
    for b in ents.values():
        types[re.match(r"\(?\s*([A-Z_0-9]+)", b).group(1)] += 1
    breps = re.findall(r"MANIFOLD_SOLID_BREP\('([^']*)'", data)
    products = re.findall(r"PRODUCT\('[^']*', '([^']*)'", data)
    print(f"{path}\n  schema={schema} entities={len(ents)} edges={len(edges)} faces={types['FACE_SURFACE']}")
    print(f"  MANIFOLD_SOLID_BREP names={breps}\n  PRODUCT names={products}")
    print(f"  refs/edge-use: {'OK' if ok else 'FAIL'}")
    return ok


if __name__ == "__main__":
    sys.exit(0 if all([lint(p) for p in sys.argv[1:]]) else 1)
