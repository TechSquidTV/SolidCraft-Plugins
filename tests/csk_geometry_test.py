#!/usr/bin/env python3
"""CSK mounting envelopes against the independent Formsprag catalog dimensions."""
import math

from harness import check, finish, package
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

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']
    for sku, bore, od, width in SIZES:
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-bearing', params={'size': sku})
        result = result.get('result', result)
        name = f'Bearing {sku} Mounting envelope'
        bodies = {body['name']: body for body in
                  host.command('inspect.measure', summary=True)['bodies']}
        body = bodies[name]
        span = [body['bbox']['max'][axis] - body['bbox']['min'][axis]
                for axis in range(3)]
        check(f'{sku}: native mounting envelope', body['closed'] and
              all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(span, (od, od, width))))
        faces = host.command('model.faces', body=name)['faces']
        radii = sorted({face['cylinder']['radius'] for face in faces
                        if 'cylinder' in face})
        check(f'{sku}: catalog bore and OD', len(radii) == 2 and
              all(math.isclose(a, b, abs_tol=1e-6)
                  for a, b in zip(radii, (bore / 2, od / 2))))
        check(f'{sku}: one body per part',
              sum(body_name.startswith(f'Bearing {sku} ') for body_name in bodies) == 1
              and not result['component']['reused'])
    rows = host.command('assembly.bom')['rows']
    check('CSK: nine distinct purchases', len(rows) == len(SIZES) and
          {row['part_number'] for row in rows} ==
          {'Formsprag ' + sku for sku, *_ in SIZES})

finish()
