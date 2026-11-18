#!/usr/bin/env python3
"""Native MISUMI exterior geometry, sourced closure detail and product reuse."""
import json
import math
from pathlib import Path

from harness import ROOT, TMP, check, finish, package
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
    ('B6000ZZ', 10, 26, 8, 6.95, 11.1, .33, 'Shield'),
    ('B6001ZZ', 12, 28, 8, 8.15, 12.45, .435, 'Shield'),
    ('B6002ZZ', 15, 32, 9, 9.65, 13.95, .435, 'Shield'),
    ('B6003ZZ', 17, 35, 10, 10.7, 15.4, .435, 'Shield'),
    ('B6004ZZ', 20, 42, 12, 13.2, 18.4, .435, 'Shield'),
    ('B6200ZZ', 10, 30, 9, 8.15, 12.45, .435, 'Shield'),
    ('B6201ZZ', 12, 32, 10, 8.7, 13.4, .435, 'Shield'),
    ('B6202ZZ', 15, 35, 11, 9.85, 15.45, .34, 'Shield'),
    ('B6203ZZ', 17, 40, 12, 11.5, 17.6, .34, 'Shield'),
    ('B6204ZZ', 20, 47, 14, 14, 20.8, .53, 'Shield'),
]:
    with Headless() as host:
        path = package('techsquid-bearings')
        approval = host.command('plugin.inspect', path=path)['approval']
        size = sku[1:].removesuffix('ZZ').removesuffix('DDU')
        style = {None:'Open','Shield':'Shielded','Seal':'Sealed'}[closure]
        def insert():
            result = host.command('plugin.run', path=path, approval=approval,
                                  entry='make-bearing', params={'size':size, 'closure':style})
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
        for ring, radii in zip(rings, [(bore/2, ri), (ro, od/2)]):
            cylinders = sorted({f['cylinder']['radius'] for f in
                                host.command('model.faces', body=ring['name'])['faces'] if 'cylinder' in f})
            check(ring['name']+': sourced bore and shoulder radii', len(cylinders) == len(radii)
                  and all(close(a,b) for a,b in zip(cylinders, radii)))
        if closure:
            # Each source fillet and shoulder must match, not just the envelope.
            # Include closure edges because the source represents one filled solid.
            section = set()
            for name in bodies:
                edges = host.command('model.edges', body=name)['edges']
                section.update((round(p[2],6), round(math.hypot(p[0],p[1]),6))
                               for edge in edges for p in (edge['start'],edge['end']) if p is not None)
            check(sku+': native section and fillets match source vertices', section ==
                  {tuple(p) for p in sources[sku+'.stp']['centered_axial_radial_vertices_mm']})
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
            # Compare actual native edges with the independently measured STEP
            # section, including where each race meets its shoulder. No builder
            # sketch names, feature IDs or production dimensions are the oracle.
            section = set()
            for ring, bottom_radius in zip(rings, [4.3, 8.2]):
                edges = host.command('model.edges', body=ring['name'])['edges']
                section.update((round(p[2],6), round(math.hypot(p[0],p[1]),6))
                               for edge in edges for p in (edge['start'],edge['end']) if p is not None)
                # The race seam crosses its deepest point at the centre plane:
                # source pitch radius 6.25 +/- groove radius 1.95 mm.
                check(ring['name']+': sourced race depth', any(
                    close(edge['mid'][2],0) and close(math.hypot(*edge['mid'][:2]),bottom_radius)
                    for edge in edges))
            check(sku+': native ring section matches source vertices', section ==
                  {tuple(p) for p in sources[sku+'.stp']['centered_axial_radial_vertices_mm']})
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

# Each new variant copies the matching measured section. Compare native solids and
# surfaces against that donor and independently check the catalog/BOM identity.
for size, closure, designation, donor, donor_id in [
    ('105', 'Sealed', '105 sealed', '105ZZ', 'openbuilds-105zz'),
    ('625', 'Shielded', 'B625ZZ', '625 2RS', 'openbuilds-625-2rs'),
    *[(str(n), 'Sealed', f'B{n}DDU', f'B{n}ZZ', f'misumi-b{n}zz')
      for n in (608, 6000, 6001, 6002, 6003, 6004, 6200, 6201, 6202, 6203, 6204)],
]:
    with Headless() as host:
        path = package('techsquid-bearings')
        approval = host.command('plugin.inspect', path=path)['approval']
        def insert(style):
            result = host.command('plugin.run', path=path, approval=approval, entry='make-bearing',
                                  params={'size':size, 'closure':style})
            return result.get('result', result)
        insert('Sealed' if closure == 'Shielded' else 'Shielded')
        result = insert(closure)
        name = 'Bearing '+designation
        definition = next(c for c in host.command('component.list')['components'] if c['name'] == name)
        meta = host.command('component.metadata', component=definition['id'])['metadata']
        evidence = meta['org.techsquid.bearing.exterior']['provenance']
        check(name+' records the estimated exterior and its measured donor',
              evidence['basis'] == 'estimated' and evidence['sources'] == [donor_id])
        bodies = {b['name']:b for b in host.command('inspect.measure',summary=True)['bodies']}
        faces = host.command('appearance.list')['bodies']
        label, other = ('Seal','Shield') if closure == 'Sealed' else ('Shield','Seal')
        for part in ('Inner ring', 'Outer ring', label+' front', label+' back'):
            b = bodies[name+' '+part]
            reference = bodies['Bearing '+donor+' '+part.replace(label, other)]
            check(name+' '+part+' matches donor geometry',
                  b['closed'] == reference['closed'] and b['face_types'] == reference['face_types']
                  and close(b['area_mm2'],reference['area_mm2'])
                  and close(b['volume_mm3'] or 0,reference['volume_mm3'] or 0)
                  and all(close(x,y) for end in ('min','max') for x,y in zip(b['bbox'][end],reference['bbox'][end])))
            if part.startswith(label):
                check(name+' '+part+' uses the selected closure colour',
                      faces[b['name']]['color'] == ('#404040' if closure == 'Sealed' else '#a0a0a0'))
        check(name+' reuses its own definition',insert(closure)['component']['reused'] and not result['component']['reused'])
        rows = host.command('assembly.bom')['rows']
        row = next(r for r in rows if r['count'] == 2)
        expected_number = None if size == '105' else 'MISUMI '+designation
        check(name+' has its own BOM row and correct purchase assignment',
              len(rows) == 2 and (row.get('part_number') or None) == expected_number
              and meta['bom_type'] == ('normal' if size == '105' else 'purchased'))

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']
    def select(params, allow_error=False):
        result = host.command('plugin.run', path=path, approval=approval, entry='make-bearing',
                              params=params, allow_error=allow_error)
        return result.get('result', result)

    default = select({})
    check('untouched dialog builds the measured B608ZZ',default['component']['component'] == 'Bearing B608ZZ')
    before = host.command('document.inspect', measure=False)
    for params, reasons in [
        ({'size':'By dimensions','match_bore':17}, ['6003 (17 × 35 × 10 mm)', '6203 (17 × 40 × 12 mm)', 'outside diameter']),
        ({'size':'By dimensions','match_bore':17,'match_diameter':41}, ['no bearing matches']),
        ({'size':'By dimensions'}, ['enter at least one']),
        ({'size':'608','closure':'Open'}, ['only for size 626']),
        ({'size':'B6203ZZ'}, ['choose a base size']),
    ]:
        error = select(params, allow_error=True).get('error','')
        check(f'{params} explains refusal without editing',all(reason in error for reason in reasons)
              and host.command('document.inspect',measure=False) == before,error)
    found = select({'size':'By dimensions','match_diameter':40,'closure':'Sealed'})
    name = found['component']['component']
    saved = str(Path(TMP)/'estimated-bearing.json')
    host.command('file.save_as',path=saved)
    host.command('doc.open',path=saved)
    again = select({'size':'6203','closure':'Sealed'})
    evidence = host.command('component.metadata',component=name)['metadata']['org.techsquid.bearing.exterior']
    check('dimension match reopens and reuses by size with its exterior evidence',
          name == 'Bearing B6203DDU' and again['component']['reused']
          and evidence['provenance']['sources'] == ['misumi-b6203zz'])

finish()
