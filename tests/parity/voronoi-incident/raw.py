"""Independent raw-byte qualification of captured Native arrays, not expected VD data."""
import struct


def verify(folder, row, rectangle=True):
    vd = row["diagram"]
    cells = list(struct.iter_unpack("<QQQ", (folder / "cells.bin").read_bytes()))
    edges = list(struct.iter_unpack("<QQQQQQ", (folder / "edges.bin").read_bytes()))
    vertices = (folder / "vertices.bin").read_bytes()
    assert len(vertices) % 32 == 0
    if rectangle:
        assert (len(cells), len(vertices) // 32, len(edges)) == (8, 5, 24)
    assert (len(cells), len(vertices) // 32, len(edges)) == (
        len(vd['cells']), len(vd['vertices']), len(vd['edges']))
    import json
    layout = json.loads((folder / 'layout.json').read_text())
    cb, eb, zb = (layout[name]['base'] for name in ['cells', 'edges', 'vertices'])
    if rectangle:
        assert (cb, eb, zb) == (min(e[0] for e in edges), min(e[2] for e in edges),
                               min(e[1] for e in edges if e[1]))
    assert sorted(e[2] for e in edges) == [eb + i * 48 for i in range(len(edges))]

    def index(pointer, base, size, count):
        if pointer == 0:
            return None
        offset = pointer - base
        assert 0 <= offset < size * count and offset % size == 0
        return offset // size

    ci = lambda p: index(p, cb, 24, len(cells))
    ei = lambda p: index(p, eb, 48, len(edges))
    vi = lambda p: index(p, zb, 32, len(vertices) // 32)
    decoded_cells = [{"source":s, "incident":ei(e), "category":c & 31,
                      "color":c >> 5} for s, e, c in cells]
    assert decoded_cells == vd["cells"]
    decoded_vertices = []
    for i in range(len(vertices) // 32):
        b = vertices[i * 32:(i + 1) * 32]
        inc, color = struct.unpack("<QQ", b[16:])
        decoded_vertices.append({"x_bits":b[:8].hex(), "y_bits":b[8:16].hex(),
                                 "incident":ei(inc), "color":color >> 5})
    assert decoded_vertices == vd["vertices"]
    decoded_edges = []
    for cell, v, twin, nxt, prev, color in edges:
        decoded_edges.append({"cell":ci(cell), "v0":vi(v), "twin":ei(twin),
                              "next":ei(nxt), "prev":ei(prev),
                              "linear":bool(color & 1), "primary":bool(color & 2),
                              "color":color >> 5})
    for i, edge in enumerate(decoded_edges):
        assert decoded_edges[edge["twin"]]["twin"] == i
        assert decoded_edges[edge["next"]]["prev"] == i
        assert decoded_edges[edge["prev"]]["next"] == i
        edge["v1"] = decoded_edges[edge["twin"]]["v0"]
        edge["finite"] = edge["v0"] is not None and edge["v1"] is not None
    assert decoded_edges == vd["edges"]
    for i, cell in enumerate(decoded_cells):
        assert decoded_edges[cell["incident"]]["cell"] == i
    for i, vertex in enumerate(decoded_vertices):
        assert decoded_edges[vertex["incident"]]["v0"] == i
    lines = []
    if row['constructor'] == 'lines':
        for a, b, c, d in struct.iter_unpack('<qqqq', (folder / 'input-lines.bin').read_bytes()):
            lines.append({'a':[a, b], 'b':[c, d]})
    else:
        for a, b, c, d, color, poly, local in struct.iter_unpack(
                "<qqqqiii4x", (folder / "colored-lines.bin").read_bytes()):
            lines.append({"a":[a, b], "b":[c, d], "color":color,
                          "poly":poly, "local_line":local})
    assert lines == row["lines"]
    if not rectangle:
        return
    # Shape and orientation identify the approved input, NOT expected diagram anchors.
    assert len(lines) == 4
    assert all(lines[i]["b"] == lines[(i + 1) % 4]["a"] for i in range(4))
    assert all(sorted(map(abs, line["a"])) == [10000000, 10000000] for line in lines)
    assert all((line["a"][0] == line["b"][0]) !=
               (line["a"][1] == line["b"][1]) for line in lines)
