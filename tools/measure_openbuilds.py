#!/usr/bin/env python3
"""Measure the supplied source revisions without copying their CAD data into git.

Usage: SOLIDCRAFT_CLI=/path/to/unsigned/solidcraft-cli python3
       tools/measure_openbuilds.py /path/to/extracted/files > measurements.json
Requires the eleven files and exact hashes in docs/sources/openbuilds-files.json.
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

ROOT = Path(__file__).resolve().parents[1]


def vertices(text):
    """Vertex inventory for the known standalone files, axis X and units mm.

    This is deliberately not a general STEP parser: placements, assembly instances
    and units are evaluated by SolidCraft below. No header data is retained.
    """
    entities = dict(re.findall(r"#(\d+)\s*=\s*(.*?);", text, re.S))
    points = []
    for entity in entities.values():
        match = re.fullmatch(r"VERTEX_POINT\('[^']*',#(\d+)\)", entity.strip())
        if not match:
            continue
        point = re.fullmatch(r"CARTESIAN_POINT\('[^']*',\((.*?)\)\)", entities[match[1]].strip(), re.S)
        if not point:
            raise ValueError("Unexpected source vertex representation")
        points.append(tuple(float(n) for n in point[1].split(',')))
    center = (min(p[0] for p in points) + max(p[0] for p in points)) / 2
    return sorted(set((round(p[0]-center, 6), round(math.hypot(p[1],p[2]), 6)) for p in points))


def main():
    directory = Path(sys.argv[1]).resolve()
    cli = os.environ.get('SOLIDCRAFT_CLI', 'solidcraft-cli')
    sources = json.loads((ROOT/'docs/sources/openbuilds-files.json').read_text())
    records = []
    with tempfile.TemporaryDirectory(prefix='openbuilds-measure-') as scratch:
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
            if len(record['bodies']) == 1:
                record['centered_axial_radial_vertices_mm'] = vertices(data.decode('utf-8'))
            records.append(record)
    print(json.dumps(records,indent=2))


if __name__ == '__main__':
    main()
