#!/usr/bin/env python3
"""Measure the supplied source revisions without copying their CAD data into git.

Usage: SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3
       tools/measure_parts.py docs/sources/<supplier>-files.json /path/to/extracted/files > measurements.json
Requires the files and exact hashes in the supplied source ledger.
No CAD application or third-party CAD library is used; SolidCraft imports STEP.
"""
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def point(entities, ref):
    match = re.fullmatch(r"CARTESIAN_POINT\s*\(\s*'[^']*',\s*\((.*?)\)\s*\)", entities[ref].strip(), re.S)
    if not match:
        raise ValueError("Unexpected source point representation")
    return tuple(float(n) for n in match[1].split(','))


def vertices(entities):
    """Vertex inventory for the known standalone files, axis X and units mm.

    This is deliberately not a general STEP parser: placements, assembly instances
    and units are evaluated by SolidCraft below. No header data is retained.
    """
    points = []
    for entity in entities.values():
        match = re.fullmatch(r"VERTEX_POINT\s*\(\s*'[^']*',\s*#(\d+)\s*\)", entity.strip())
        if not match:
            continue
        points.append(point(entities, match[1]))
    center = (min(p[0] for p in points) + max(p[0] for p in points)) / 2
    return sorted(set((round(p[0]-center, 6), round(math.hypot(p[1],p[2]), 6)) for p in points))


def analytic_surfaces(text):
    """Record source radii without retaining header paths or author metadata."""
    surfaces = set()
    for kind, radii in re.findall(
            r"(CYLINDRICAL|TOROIDAL|SPHERICAL)_SURFACE\s*\(\s*'[^']*',\s*#\d+,\s*([^;]*?)\)", text):
        surfaces.add((kind.lower(), tuple(round(float(n), 6) for n in radii.split(','))))
    return [{'kind':kind, 'radii_mm':radii} for kind,radii in sorted(surfaces)]


def spheres(entities):
    records = []
    for entity in entities.values():
        match = re.fullmatch(r"SPHERICAL_SURFACE\s*\(\s*'[^']*',\s*#(\d+),\s*([\d.E+-]+)\s*\)", entity.strip())
        if match:
            placement = entities[match[1]]
            origin = re.findall(r'#(\d+)', placement)[0]
            records.append({'center_mm':[round(n,6) for n in point(entities, origin)],
                            'radius_mm':round(float(match[2]),6)})
    return sorted(records, key=lambda r:r['center_mm'])


def solid_sections(entities):
    """Local axial/radial vertices per solid in the supplied X-axis CSK assemblies.

    Assembly placement is evaluated by the native body inventory, not guessed
    here. In particular, cover axial offsets come from the native bounding boxes.
    """
    sections = []
    for ref, entity in entities.items():
        if not entity.strip().startswith('MANIFOLD_SOLID_BREP'):
            continue
        visited, pending = set(), [ref]
        while pending:
            current = pending.pop()
            if current in visited:
                continue
            visited.add(current)
            pending.extend(re.findall(r'#(\d+)', entities[current]))
        subset = {key: entities[key] for key in visited}
        sections.append({
            'solid_id': int(ref),
            'centered_axial_radial_vertices_mm': vertices(subset),
            'analytic_surfaces': analytic_surfaces('\n'.join(subset.values())),
        })
    return sections


def main():
    ledger = Path(sys.argv[1]).resolve()
    directory = Path(sys.argv[2]).resolve()
    cli = os.environ.get('SOLIDCRAFT_CLI', 'solidcraft-cli')
    sources = json.loads(ledger.read_text())
    records = []
    with tempfile.TemporaryDirectory(prefix='parts-measure-') as scratch:
        script, output = Path(scratch)/'script.json', Path(scratch)/'result.json'
        for source in sources:
            path = directory/source['file']
            data = path.read_bytes()
            if hashlib.sha256(data).hexdigest() != source['sha256']:
                raise ValueError('Source revision changed: '+source['file'])
            script.write_text(json.dumps([
                {'command':'file.insert_step','params':{'path':str(path),'reuse_import':False}},
                {'command':'inspect.measure','params':{'summary':True}},
            ]))
            subprocess.run([cli,'run',str(script),'--quiet','--results','--json-out',str(output)],
                           check=True, capture_output=True, text=True)
            results = json.loads(output.read_text())
            measure = results[-1].get('result', results[-1])
            record = {'file':source['file'], 'sha256':source['sha256'], 'bodies':[
                {k:body[k] for k in ['name','closed','bbox','volume_mm3','face_types']}
                for body in measure['bodies']
            ]}
            text = data.decode('utf-8')
            entities = dict(re.findall(r"#(\d+)\s*=\s*(.*?);", text, re.S))
            if len(record['bodies']) == 1:
                record['centered_axial_radial_vertices_mm'] = vertices(entities)
                record['analytic_surfaces'] = analytic_surfaces(text)
                if measured := spheres(entities):
                    record['spheres'] = measured
            else:
                record['source_solid_sections'] = solid_sections(entities)
            records.append(record)
    print(json.dumps(records,indent=2))


if __name__ == '__main__':
    main()
