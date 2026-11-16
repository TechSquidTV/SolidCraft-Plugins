#!/usr/bin/env python3
"""Native migration checks: representation reuse, purchasing BOM and edits."""

from harness import check, finish, package
from headless import Headless

CASES = [
    ('techsquid-screws', 'make-screw', {'size': 'M3', 'create': 'Screw only'}, {'head_diameter': 5.5}),
    ('techsquid-nuts', 'make-nut', {'size': 'M6', 'create': 'Nut only'}, {'custom_width': 10}),
    ('techsquid-inserts', 'make-insert', {'size': 'M3 × 5.7', 'create': 'Insert only'}, {'custom_length': 5.7}),
    ('techsquid-extrusions', 'make-extrusion', {'slot': 'V-slot', 'size': '2020', 'length': 40, 'end_taps': 'Start'}, {'length': 50}),
    ('techsquid-extrusions', 'make-t-nut', {'type': 'Pre-assembly', 'size': 'M5'}, None),
]

for plugin, entry, params, custom in CASES:
    with Headless() as host:
        approval = host.command('plugin.inspect', path=package(plugin))['approval']

        def insert(**overrides):
            result = host.command('plugin.run', path=package(plugin), approval=approval,
                                  entry=entry, params={**params, **overrides})
            return result.get('result', result)

        def definition(name):
            return next(c for c in host.command('component.list')['components'] if c['name'] == name)

        def metadata(component):
            return host.command('component.metadata', component=component)['metadata']

        label = f'{plugin}/{entry}'
        cosmetic = insert(thread_modeled=False)
        modeled = insert(thread_modeled=True)
        a = definition(cosmetic['component']['component'])
        b = definition(modeled['component']['component'])
        ma, mb = metadata(a['id']), metadata(b['id'])
        ra, rb = ma['org.techsquid.parts'], mb['org.techsquid.parts']
        check(f'{label}: detail modes retain one purchasing identity',
              a['id'] != b['id'] and ra['product'] == rb['product'] and ra['representation'] != rb['representation'])
        check(f'{label}: each representation reuses its verified definition',
              insert(thread_modeled=False)['component']['reused'] and insert(thread_modeled=True)['component']['reused'])
        rows = host.command('assembly.bom')['rows']
        check(f'{label}: BOM combines detail modes without double counting',
              len(rows) == 1 and rows[0]['count'] == 4 and len(rows[0]['components']) == 2)
        host.command('component.rename', component=a['id'], name='Renamed catalog part')
        renamed = insert(thread_modeled=False)
        check(f'{label}: display rename retains reuse', renamed['component']['reused'] and renamed['component']['component'] == 'Renamed catalog part')
        body = a['bodies'][0]
        host.command('appearance.assign', bodies=[body], color=[240, 10, 20])
        fresh = insert(thread_modeled=False)
        check(f'{label}: edited appearance is preserved and a clean definition is built',
              not fresh['component']['reused'] and fresh['library']['preserved_modified_definitions'][0]['component'] == a['id']
              and host.command('appearance.list')['bodies'][body]['color'] == '#f00a14')
        check(f'{label}: unchanged detail remains reusable after its sibling was edited', insert(thread_modeled=True)['component']['reused'])
        if custom:
            changed = insert(thread_modeled=False, **custom)
            c = definition(changed['component']['component'])
            mc = metadata(c['id'])
            check(f'{label}: changed product has a distinct BOM number',
                  mc['org.techsquid.parts']['product'] != ra['product']
                  and mc['part_number'] != ma['part_number'] and mc['bom_type'] == 'normal')
        if entry == 'make-t-nut':
            mounted = insert(thread_modeled=False, slot='V-slot', along='Sketch Y')
            check(f'{label}: mounting orientation does not change the purchased nut', mounted['component']['reused'])
        check(f'{label}: insertion stays within the command budget',
              max(cosmetic['library']['host_calls'], modeled['library']['host_calls'], fresh['library']['host_calls']) < 248)

finish()
