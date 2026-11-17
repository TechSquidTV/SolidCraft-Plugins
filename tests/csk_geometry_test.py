#!/usr/bin/env python3
"""CSK mounting envelopes against the independent Formsprag catalog dimensions."""
import math

from harness import TMP, check, finish, package
from pathlib import Path
from headless import Headless


# Formsprag P-1352-FC, page 3; not read from the compiled plugin catalog.
SIZES = [
    ('CSK8', 8, 22, 9),
    ('CSK12', 12, 32, 10),
    ('CSK15', 15, 35, 11),
    ('CSK17', 17, 40, 12),
    ('CSK20', 20, 47, 14),
    ('CSK25', 25, 52, 15),
    ('CSK30', 30, 62, 16),
    ('CSK35', 35, 72, 17),
    ('CSK40', 40, 80, 22),
]

# Keyways independently transcribed from Formsprag P-1352-FC p4 and
# MCC-P-956-FC-EN-US 11/25 pp28–29. Depths are from the bore/OD apex.
KEYS = {12: (4, 1.8, None, None), 15: (5, 1.2, 2, .6),
        17: (5, 1.2, 2, 1), 20: (6, 1.6, 3, 1.5),
        25: (8, 2, 6, 2), 30: (8, 2, 6, 2),
        35: (10, 2.4, 8, 2.5), 40: (12, 3.3, 10, 3)}
CASES = []
for sku, bore, od, width in SIZES:
    sealed_width = 9 if bore == 8 else 14 if bore == 12 else width + 5
    for suffix, sealed, keyed, outer in [('', False, False, False),
            ('-2RS', True, False, False), ('P', False, True, False),
            ('PP', False, True, True), ('P-2RS', True, True, False),
            ('PP-2RS', True, True, True)]:
        if keyed and bore == 8 or outer and bore == 12:
            continue
        if suffix == 'PP-2RS' and bore != 30:
            continue
        CASES.append((sku + suffix, bore, od, sealed_width if sealed else width, keyed, outer))

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']
    for sku, bore, od, width, keyed, outer in CASES:
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-csk', params={'size': sku})
        result = result.get('result', result)
        name = f'CSK clutch {sku} Mounting envelope'
        bodies = {body['name']: body for body in
                  host.command('inspect.measure', summary=True)['bodies']}
        body = bodies[name]
        span = [body['bbox']['max'][axis] - body['bbox']['min'][axis]
                for axis in range(3)]
        check(f'{sku}: native mounting envelope', body['closed'] and
              all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(span, (od, od / 2 + math.sqrt((od / 2)**2 - (KEYS[bore][2] / 2)**2) if outer else od, width))))
        faces = host.command('model.faces', body=name)['faces']
        radii = sorted({face['cylinder']['radius'] for face in faces
                        if 'cylinder' in face})
        check(f'{sku}: catalog bore and OD', len(radii) == 2 and
              all(math.isclose(a, b, abs_tol=1e-6)
                  for a, b in zip(radii, (bore / 2, od / 2))))
        if keyed:
            edges = host.command('model.edges', body=name)['edges']
            vertices = [p for edge in edges for p in (edge['start'], edge['end']) if p is not None]
            iw, depth, ow, outer_depth = KEYS[bore]
            slots = [(iw, bore / 2 + depth)]
            if outer:
                slots.append((ow, od / 2 - outer_depth))
            for key_width, floor in slots:
                expected = [(x, floor, z) for x in (-key_width / 2, key_width / 2)
                            for z in (-width / 2, width / 2)]
                check(f'{sku}: through keyway width and floor {floor}', all(
                    any(all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(p, target))
                        for p in vertices) for target in expected))
        check(f'{sku}: one body per part',
              sum(body_name.startswith(f'CSK clutch {sku} ') for body_name in bodies) == 1
              and not result['component']['reused'])
    rows = host.command('assembly.bom')['rows']
    check('CSK: 42 distinct unsourced specifications', len(rows) == len(CASES) and
          all(not row.get('part_number') and row['count'] == 1 for row in rows))

    for invalid in ['CSK8P', 'CSK12PP', 'CSK20PP-2RS']:
        rejected = host.command('plugin.run', path=path, approval=approval,
                                entry='make-csk', params={'size': invalid}, allow_error=True)
        check(invalid + ': unsourced combination rejected', 'error' in rejected)

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']
    def insert(**params):
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-csk', params={'size': 'CSK12P', **params})
        return result.get('result', result)
    def definition(result):
        return next(c for c in host.command('component.list')['components']
                    if c['name'] == result['component']['component'])
    default = insert()
    custom = insert(inner_keyway_depth=1.2)
    repeated = insert(inner_keyway_depth=1.2)
    component = definition(custom)
    record = host.command('component.metadata', component=component['id'])['metadata']
    spec = record['org.techsquid.specification']
    check('custom: geometry differs from reference without assigning supplier',
          spec['geometry']['inner_keyway'] == {'width': 4.0, 'depth': 1.2}
          and [spec['geometry'][k] for k in ('bore','diameter','width')] == [12,32,10]
          and spec['reference']['baseline']['inner_keyway']['depth'] == 1.8
          and spec['reference']['customized'] and 'Formsprag' in spec['reference']['title']
          and 'purchasing' not in spec and 'purchasing_identity' not in spec
          and not record.get('part_number'))
    check('custom: reference vendor is absent from canonical design identity',
          'formsprag' not in record['org.techsquid.parts']['product'].lower())
    check('custom: identical overrides reuse; different geometry stays distinct',
          repeated['component']['reused'] and
          default['component']['component'] != custom['component']['component'])
    body = component['bodies'][0]
    # component.list returns body names; compare the native slot floor directly.
    edges = host.command('model.edges', body=body)['edges']
    vertices = [p for edge in edges for p in (edge['start'], edge['end']) if p is not None]
    check('custom: native slot is 1.2 mm deep through the 10 mm width', all(
        any(all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(p, target)) for p in vertices)
        for target in [(-2,7.2,-5),(2,7.2,-5),(-2,7.2,5),(2,7.2,5)]))
    rows = host.command('assembly.bom')['rows']
    check('custom: unsourced specifications count separately in BOM',
          len(rows) == 2 and sorted(row['count'] for row in rows) == [1,2]
          and all(not row.get('part_number') for row in rows))
    saved = str(Path(TMP) / 'custom-csk.sc')
    host.command('file.save_as', path=saved)
    host.command('doc.open', path=saved)
    reopened = insert(inner_keyway_depth=1.2)
    check('custom: geometry, references and reuse survive reopen',
          reopened['component']['reused'] and definition(reopened)['id'] == component['id']
          and host.command('component.metadata', component=component['id'])['metadata'] == record)
    before = host.command('document.inspect', measure=False)
    for params in [{'inner_keyway_width': 12}, {'inner_keyway_depth': 20}, {'outer_keyway_depth': 1},
                   {'part_number': 'CL-12'}, {'purchase_supplier': 'Seller'}]:
        result = host.command('plugin.run', path=path, approval=approval, entry='make-csk',
                              params={'size': 'CSK12P', **params}, allow_error=True)
        check('custom: invalid keyway fails without edits', 'error' in result
              and host.command('document.inspect', measure=False) == before)

finish()
