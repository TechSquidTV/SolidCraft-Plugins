#!/usr/bin/env python3
"""Native MISUMI exterior geometry, sourced closure detail and product reuse."""
import json
import math
from pathlib import Path

from harness import ROOT, check, finish, package
from headless import Headless

sources = {s['file']: s for s in json.loads((Path(ROOT)/'docs/sources/misumi-measurements.json').read_text())}

def close(a, b):
    return math.isclose(a, b, rel_tol=1e-8, abs_tol=1e-6)

# Independent supplier dimensions; do not load the compiled catalog as the oracle.
for sku, bore, od, width, ri, ro, recess, closure in [
    ('B626', 6, 19, 6, 5.06, 7.55, None, None),
    ('B626ZZ', 6, 19, 6, 4.75, 8.15, .5, 'Shield'),
    ('B626DDU', 6, 19, 6, 4.215, 8.1775, .11, 'Seal'),
    ('B608ZZ', 8, 22, 7, 6.1, 9.65, .6, 'Shield'),
]:
    with Headless() as host:
        path = package('techsquid-bearings')
        approval = host.command('plugin.inspect', path=path)['approval']
        def insert():
            result = host.command('plugin.run', path=path, approval=approval,
                                  entry='make-bearing', params={'size':sku})
            return result.get('result', result)
        first = insert()
        component = next(c for c in host.command('component.list')['components']
                         if c['name'] == first['component']['component'])
        metadata = host.command('component.metadata', component=component['id'])['metadata']
        check(sku+': exact supplier purchase identity', metadata['part_number'] == 'MISUMI '+sku
              and metadata['bom_type'] == 'purchased'
              and json.loads(metadata['org.techsquid.parts']['product'])['catalog'] == 'misumi')
        bodies = {b['name']:b for b in host.command('inspect.measure', summary=True)['bodies']}
        rings = [bodies['Bearing '+sku+' '+label] for label in ('Inner ring', 'Outer ring')]
        check(sku+': two native rings and only sourced closures', len(bodies) == (4 if closure else 10)
              and all(b['closed'] and b['face_types'].get('sphere',0) == 0 for b in rings))
        check(sku+': sourced envelope and rounded edges', all(close(a,b) for a,b in zip(
            [rings[1]['bbox']['max'][i]-rings[1]['bbox']['min'][i] for i in range(3)], [od,od,width]))
            and all(b['face_types'].get('other') == (2 if closure else 3) for b in rings))
        if closure:
            source_volume = sources[sku+'.stp']['bodies'][0]['volume_mm3']
            # The supplier STEP fills the space between closure faces. Removing
            # that known annular fill leaves the independently measured ring volume.
            ring_volume = source_volume - math.pi*(ro**2-ri**2)*(width-2*recess)
            check(sku+': curved ring volume agrees with source STEP',
                  close(sum(b['volume_mm3'] for b in rings), ring_volume))
            appearance = host.command('appearance.list')['bodies']
            for side, sign in [('front',1), ('back',-1)]:
                name = 'Bearing '+sku+' '+closure+' '+side
                body = bodies[name]
                z = sign*(width/2-recess)
                check(name+': exact closure area, recess and appearance', not body['closed']
                      and body['volume_mm3'] is None and close(body['area_mm2'], math.pi*(ro**2-ri**2))
                      and close(body['bbox']['min'][2],z) and close(body['bbox']['max'][2],z)
                      and appearance[name]['color'] == ('#404040' if closure == 'Seal' else '#a0a0a0'))
        else:
            balls = [bodies['Bearing '+sku+' Ball '+str(i+1)] for i in range(8)]
            for i, ball in enumerate(balls):
                angle = math.tau*i/8
                expected = [6.25*math.cos(angle), 6.25*math.sin(angle), 0]
                center = [(ball['bbox']['min'][j]+ball['bbox']['max'][j])/2 for j in range(3)]
                check(sku+f': ball {i+1} matches the sourced radius and pitch circle', ball['closed']
                      and ball['face_types'].get('sphere') == 1
                      and close(ball['volume_mm3'], 4/3*math.pi*1.975**3)
                      and all(close(a,b) for a,b in zip(center, expected)))
        second = insert()
        check(sku+': curved section survives native verification and reuses', second['component']['reused']
              and second['component']['component'] == component['name'])
        rows = host.command('assembly.bom')['rows']
        check(sku+': one purchase per occurrence', len(rows) == 1
              and rows[0]['part_number'] == 'MISUMI '+sku and rows[0]['count'] == 2)
        check(sku+': commands remain within budget', first['library']['host_calls'] < 248)

finish()
