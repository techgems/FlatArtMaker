"""Throwaway spike (task 02): write two-part PrusaSlicer-dialect 3MF variants.

One object, two parts: a 40x40x2 mm `Base` on filament 1 and a 20x20x1 mm
`Black` square on top of it on filament 2. Variants drop optional pieces so the
round-trip check can show which ones the slicers actually need.

Usage: python make_3mf.py <out_dir>
"""

import sys
import zipfile
from pathlib import Path

CONTENT_TYPES = """<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
 <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
 <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>
"""

RELS = """<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
 <Relationship Target="/3D/3dmodel.model" Id="rel-1" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>
"""


def box(x0, y0, z0, x1, y1, z1):
    """Closed box: 8 vertices, 12 outward-facing (CCW) triangles."""
    v = [
        (x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0),
        (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1),
    ]
    t = [
        (0, 2, 1), (0, 3, 2),  # bottom (-z)
        (4, 5, 6), (4, 6, 7),  # top (+z)
        (0, 1, 5), (0, 5, 4),  # -y
        (1, 2, 6), (1, 6, 5),  # +x
        (2, 3, 7), (2, 7, 6),  # +y
        (3, 0, 4), (3, 4, 7),  # -x
    ]
    return v, t


PARTS = [
    # name, extruder, box
    ("Base", 1, box(0, 0, 0, 40, 40, 2)),
    ("Black", 2, box(10, 10, 2, 30, 30, 3)),
]


def model_xml(with_version: bool, with_colorgroup: bool) -> tuple[str, list]:
    verts, tris, ranges = [], [], []
    for _, _, (v, t) in PARTS:
        base = len(verts)
        first = len(tris)
        verts += v
        tris += [(a + base, b + base, c + base) for a, b, c in t]
        ranges.append((first, len(tris) - 1))
    out = ['<?xml version="1.0" encoding="UTF-8"?>']
    ns = ' xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"' if with_colorgroup else ""
    out.append(
        '<model unit="millimeter" xml:lang="en-US"'
        ' xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"'
        f' xmlns:slic3rpe="http://schemas.slic3r.org/3mf/2017/06"{ns}>'
    )
    if with_version:
        out.append(' <metadata name="slic3rpe:Version3mf">1</metadata>')
    out.append(" <resources>")
    if with_colorgroup:
        out.append('  <m:colorgroup id="2"><m:color color="#FFFFFFFF"/><m:color color="#000000FF"/></m:colorgroup>')
    out.append('  <object id="1" type="model"><mesh><vertices>')
    out += [f'   <vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in verts]
    out.append("  </vertices><triangles>")
    for i, (a, b, c) in enumerate(tris):
        attrs = ""
        if with_colorgroup:
            part = next(p for p, (f, l) in enumerate(ranges) if f <= i <= l)
            attrs = f' pid="2" p1="{part}"'
        out.append(f'   <triangle v1="{a}" v2="{b}" v3="{c}"{attrs}/>')
    out.append("  </triangles></mesh></object>")
    out.append(" </resources>")
    out.append(' <build><item objectid="1" transform="1 0 0 0 1 0 0 0 1 100 100 0" printable="1"/></build>')
    out.append("</model>")
    return "\n".join(out) + "\n", ranges


def config_xml(ranges, full_keys: bool) -> str:
    out = ['<?xml version="1.0" encoding="UTF-8"?>', "<config>", ' <object id="1" instances_count="1">']
    out.append('  <metadata type="object" key="name" value="Two squares"/>')
    for (name, extruder, _), (first, last) in zip(PARTS, ranges):
        out.append(f'  <volume firstid="{first}" lastid="{last}">')
        out.append(f'   <metadata type="volume" key="name" value="{name}"/>')
        if full_keys:
            out.append('   <metadata type="volume" key="volume_type" value="ModelPart"/>')
            out.append('   <metadata type="volume" key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>')
        out.append(f'   <metadata type="volume" key="extruder" value="{extruder}"/>')
        out.append("  </volume>")
    out += [" </object>", "</config>", ""]
    return "\n".join(out)


VARIANTS = {
    # name: (slic3rpe:Version3mf, volume_type+matrix keys, m:colorgroup)
    "full": (True, True, True),
    "no-colorgroup": (True, True, False),
    "minimal": (True, False, False),
    "no-version": (False, False, False),
}


def write(path: Path, with_version: bool, full_keys: bool, with_colorgroup: bool) -> None:
    model, ranges = model_xml(with_version, with_colorgroup)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", CONTENT_TYPES)
        z.writestr("_rels/.rels", RELS)
        z.writestr("3D/3dmodel.model", model)
        z.writestr("Metadata/Slic3r_PE_model.config", config_xml(ranges, full_keys))


# --- Bambu Studio dialect -------------------------------------------------
# One mesh object per part, assembled as components of a parent object; part
# names and filaments live in Metadata/model_settings.config keyed by part id.

BAMBU_NS = (
    'xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"'
    ' xmlns:BambuStudio="http://schemas.bambulab.com/package/2021"'
    ' xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"'
)


def mesh_object(obj_id: int, v, t) -> str:
    rows = [f'  <object id="{obj_id}" type="model"><mesh><vertices>']
    rows += [f'   <vertex x="{x}" y="{y}" z="{z}"/>' for x, y, z in v]
    rows.append("  </vertices><triangles>")
    rows += [f'   <triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in t]
    rows.append("  </triangles></mesh></object>")
    return "\n".join(rows)


def bambu_files(split: bool, with_version: bool, with_app: bool) -> dict[str, str]:
    parent = len(PARTS) + 1
    meta = []
    if with_app:
        meta.append(' <metadata name="Application">BambuStudio-02.03.06.00</metadata>')
    if with_version:
        meta.append(' <metadata name="BambuStudio:3mfVersion">1</metadata>')
    meshes = "\n".join(mesh_object(i + 1, v, t) for i, (_, _, (v, t)) in enumerate(PARTS))
    path_attr = ' p:path="/3D/Objects/object_1.model"' if split else ""
    comps = "\n".join(
        f'    <component{path_attr} objectid="{i + 1}" transform="1 0 0 0 1 0 0 0 1 0 0 0"/>'
        for i in range(len(PARTS))
    )
    req = ' requiredextensions="p"' if split else ""
    main = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<model unit="millimeter" xml:lang="en-US" {BAMBU_NS}{req}>',
        *meta,
        " <resources>",
        *([] if split else [meshes]),
        f'  <object id="{parent}" type="model"><components>',
        comps,
        "  </components></object>",
        " </resources>",
        f' <build><item objectid="{parent}" transform="1 0 0 0 1 0 0 0 1 100 100 0" printable="1"/></build>',
        "</model>",
        "",
    ]
    settings = ['<?xml version="1.0" encoding="UTF-8"?>', "<config>", f'  <object id="{parent}">']
    settings.append('    <metadata key="name" value="Two squares"/>')
    settings.append('    <metadata key="extruder" value="1"/>')
    for i, (name, extruder, _) in enumerate(PARTS):
        settings.append(f'    <part id="{i + 1}" subtype="normal_part">')
        settings.append(f'      <metadata key="name" value="{name}"/>')
        settings.append(f'      <metadata key="extruder" value="{extruder}"/>')
        settings.append("    </part>")
    settings += ["  </object>", "</config>", ""]
    files = {
        "[Content_Types].xml": CONTENT_TYPES,
        "_rels/.rels": RELS,
        "3D/3dmodel.model": "\n".join(main),
        "Metadata/model_settings.config": "\n".join(settings),
    }
    if split:
        files["3D/_rels/3dmodel.model.rels"] = (
            '<?xml version="1.0" encoding="UTF-8"?>\n'
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">\n'
            ' <Relationship Target="/3D/Objects/object_1.model" Id="rel-1"'
            ' Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>\n'
            "</Relationships>\n"
        )
        files["3D/Objects/object_1.model"] = "\n".join(
            [
                '<?xml version="1.0" encoding="UTF-8"?>',
                f'<model unit="millimeter" xml:lang="en-US" {BAMBU_NS} requiredextensions="p">',
                " <resources>",
                meshes,
                " </resources>",
                " <build/>",
                "</model>",
                "",
            ]
        )
    return files


BAMBU_VARIANTS = {
    # name: (split into 3D/Objects, BambuStudio:3mfVersion, Application)
    "bambu-split": (True, True, True),
    "bambu-inline": (False, True, True),
    "bambu-no-app": (False, True, False),
    "bambu-no-version": (False, False, True),
}


def write_bambu(path: Path, split: bool, with_version: bool, with_app: bool) -> None:
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in bambu_files(split, with_version, with_app).items():
            z.writestr(name, data)


def write_hybrid(path: Path) -> None:
    """Bambu component layout plus Prusa's version tag and per-volume config."""
    files = bambu_files(split=False, with_version=True, with_app=False)
    files["3D/3dmodel.model"] = files["3D/3dmodel.model"].replace(
        " <resources>", ' <metadata name="slic3rpe:Version3mf">1</metadata>\n <resources>', 1
    ).replace(BAMBU_NS, BAMBU_NS + ' xmlns:slic3rpe="http://schemas.slic3r.org/3mf/2017/06"', 1)
    ranges, first = [], 0
    for _, _, (_, t) in PARTS:
        ranges.append((first, first + len(t) - 1))
        first += len(t)
    files["Metadata/Slic3r_PE_model.config"] = config_xml(ranges, full_keys=False).replace(
        '<object id="1"', f'<object id="{len(PARTS) + 1}"', 1
    )
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in files.items():
            z.writestr(name, data)


if __name__ == "__main__":
    out_dir = Path(sys.argv[1])
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, flags in VARIANTS.items():
        write(out_dir / f"two-squares-{name}.3mf", *flags)
        print(out_dir / f"two-squares-{name}.3mf")
    for name, flags in BAMBU_VARIANTS.items():
        write_bambu(out_dir / f"two-squares-{name}.3mf", *flags)
        print(out_dir / f"two-squares-{name}.3mf")
    write_hybrid(out_dir / "two-squares-hybrid.3mf")
    print(out_dir / "two-squares-hybrid.3mf")
