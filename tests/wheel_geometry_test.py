#!/usr/bin/env python3
"""Check native wheel/bearing geometry against measured supplier reference files.

Build first: cargo build -p wheels --example reference_geometry
             cargo build -p plugin-kit --example inspect_content
Uses the checked-in measurement ledger; source STEP files are not needed in CI.
"""
import json
import math
import os
from pathlib import Path
import subprocess

from harness import CLI, ROOT, TMP, check, finish, inspect, package, run

TARGET = Path(os.environ.get('CARGO_TARGET_DIR', Path(ROOT)/'target'))/'debug/examples'
steps = json.loads(subprocess.check_output([str(TARGET/'reference_geometry')], text=True))
design = os.path.join(TMP, 'reference.sc')
code, text, results = run(steps, save=design)
check('all wheel, bearing and shim definitions build', code == 0, text[-1000:] if code else '')
if code:
    finish()
bodies = {b['name']: b for b in results[-3]['bodies']}
components = {c['name']: c for c in results[-2]['components']}
appearance = results[-1]['bodies']
source = {r['file']: r for r in json.loads((Path(ROOT)/'docs/sources/openbuilds-measurements.json').read_text())}


def size(body):
    return [b-a for a,b in zip(body['bbox']['min'], body['bbox']['max'])]


def close(actual, expected):
    return math.isclose(actual, expected, abs_tol=1e-6, rel_tol=1e-8)


for name in ['Solid V Wheel', 'Xtreme Solid V Wheel', 'Delrin V Wheel', 'Delrin Mini V Wheel', 'Xtreme Mini V Wheel']:
    actual, ref = bodies[name], source[name+'.step']['bodies'][0]
    check(name+': solid volume matches STEP', actual['closed'] and close(actual['volume_mm3'], ref['volume_mm3']))
    ref_size = size(ref)
    # Standalone references revolve on X; our axle is Z.
    check(name+': bounds match STEP after axis conversion', all(close(a,b) for a,b in zip(size(actual), [ref_size[1],ref_size[2],ref_size[0]])))
    check(name+': original native profile, no imported body', len(components[name]['features']) == 1 and len(components[name]['sketches']) == 1)

for name, inner_radius, ring_inner, ring_outer, outer_radius, width, closure, label in [
    ('625 2RS', 2.5,4.3,6.8,8.0,5.0,2.2,'Seal'),
    ('105ZZ', 2.5,3.1,4.6,5.0,4.0,1.7,'Shield'),
]:
    check(name+': two rings and two closures, no balls', len(components[name]['bodies']) == 4)
    for suffix, ri, ro in [('Inner ring', inner_radius, ring_inner), ('Outer ring', ring_outer, outer_radius)]:
        body = bodies[name+' '+suffix]
        check(name+' '+suffix+': exact radial envelope', body['closed'] and close(body['volume_mm3'], math.pi*(ro**2-ri**2)*width)
              and all(close(a,b) for a,b in zip(size(body), [2*ro,2*ro,width])))
    for side, z in [('front',closure), ('back',-closure)]:
        key = name+' '+label+' '+side
        body = bodies[key]
        check(key+': closure face has sourced recess, no invented thickness', not body['closed'] and body['volume_mm3'] is None
              and close(body['bbox']['min'][2],z) and close(body['bbox']['max'][2],z)
              and close(body['area_mm2'],math.pi*(ring_outer**2-ring_inner**2)))
        check(key+': appropriate closure appearance', appearance[key]['color'] == ('#404040' if label == 'Seal' else '#a0a0a0'))

for sku, od, expected_volume in [('05',10,58.90486225480861), ('170',8,30.630528372500482)]:
    body = bodies['Shim '+sku]
    check('Shim '+sku+': correct nominal spacer', body['closed'] and close(body['volume_mm3'],expected_volume)
          and all(close(a,b) for a,b in zip(size(body),[od,od,1])))

check('material variants keep distinct appearances', appearance['Solid V Wheel']['opacity'] == 1
      and appearance['Xtreme Solid V Wheel']['opacity'] < 1)

code, text, reopened = run([{'command':'inspect.measure','params':{'summary':True}}], inp=design)
check('all geometry survives save/reopen', code == 0 and reopened[0]['bodies'] == results[-3]['bodies'], text[-1000:] if code else '')

for name in ['Solid V Wheel', 'Delrin V Wheel', 'Delrin Mini V Wheel', '625 2RS', '105ZZ', 'Shim 05', 'Shim 170']:
    reply = subprocess.run([str(TARGET/'inspect_content'), CLI, design, str(components[name]['id'])], capture_output=True, text=True)
    check(name+': current geometry can be independently inspected for reuse', reply.returncode == 0, reply.stderr[-500:])

# The V-slot top chamfers run from (3.125,8.545) to (4.58,10),
# y = abs(x) + 5.42. The wheel's outward 45-degree flanks have r+abs(z)
# 14.9 (solid), 14.92 (grooved), 10.505 (mini). Their sums with 5.42
# independently determine the axle positions; these are not fitted to a test run.
wheel_names = ['Solid V Wheel','Xtreme Solid V Wheel','Delrin V Wheel','Delrin Mini V Wheel','Xtreme Mini V Wheel']
offsets = [20.32,20.32,20.34,15.925,15.925]
code, text, inspected = inspect('techsquid-extrusions')
check('rail package is available for contact check', code == 0, text[-1000:] if code else '')
if not code:
    contact_design = os.path.join(TMP,'wheel-contact.sc')
    steps = [{'command':'plugin.run','params':{'path':package('techsquid-extrusions'), 'approval':inspected['approval'],
              'entry':'make-extrusion','params':{'slot':'V-slot','size':'2020','length':220}}}]
    for i,(name,height) in enumerate(zip(wheel_names,offsets)):
        occurrence = components[name]['occurrences'][0]['id']
        # Original wheel axis +Z becomes world +X; radial X becomes +Y.
        steps.append({'command':'occurrence.move','params':{'occurrence':occurrence,
                      'to_matrix':[[0,0,1,0],[1,0,0,height-30*i],[0,1,0,30+35*i]]}})
    clearance = {'command':'inspect.clearance','params':{'bodies':wheel_names,'against':['V-slot 2020 x 220'],
                 'clearance':0.1,'min_volume':0.000001}}
    code, text, contact = run(steps+[clearance],inp=design,save=contact_design)
    check('all wheel axes place across the rail',code == 0,text[-1000:] if code else '')
    if not code:
        result = contact[-1]
        check('five exact wheel/rail contacts with no overlap or unchecked pair', result['pairs_checked'] == 5
              and not result['unchecked'] and not result['overlaps'] and not result['truncated']
              and len(result['near']) == 5 and all(p['distance_mm'] == 0 for p in result['near']))
        for label,dy in [('gap',0.05),('penetration',-0.05)]:
            moved = [{'command':'occurrence.move','params':{'occurrence':components[name]['occurrences'][0]['id'],
                      'translate':[0,dy,0]}} for name in wheel_names]
            code, text, result = run(moved+[clearance],inp=contact_design)
            check(label+': controlled displacement succeeds',code == 0,text[-1000:] if code else '')
            if code:
                continue
            result = result[-1]
            if label == 'gap':
                check('lifting every wheel opens the expected diagonal clearance',not result['overlaps'] and not result['unchecked']
                      and len(result['near']) == 5 and all(close(p['distance_mm'],0.05/math.sqrt(2)) for p in result['near']))
            else:
                check('lowering every wheel produces actual interference',len(result['overlaps']) == 5 and not result['unchecked']
                      and all(p['volume_mm3'] > 0.000001 for p in result['overlaps']))

finish()
