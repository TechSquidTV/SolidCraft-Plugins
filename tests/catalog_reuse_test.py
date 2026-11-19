#!/usr/bin/env python3
"""Native migration checks: representation reuse, purchasing BOM and edits."""

import json

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
        # Field labels and body names are not parameter dependencies.
        host.command('parameters.add', name='radius', expression=17)
        check(f'{label}: unrelated parameter has no native users',
              host.command('parameters.users', name='radius')['drives'] == [])
        for value in (17, 21):
            host.command('parameters.change', name='radius', expression=value)
            unrelated = insert(thread_modeled=False)
            check(f'{label}: unrelated radius={value} preserves reuse',
                  unrelated['component']['reused']
                  and unrelated['component']['component'] == a['name']
                  and not unrelated['library']['preserved_modified_definitions'])
        host.command('component.rename', component=a['id'], name='Renamed catalog part')
        renamed = insert(thread_modeled=False)
        check(f'{label}: display rename retains reuse', renamed['component']['reused'] and renamed['component']['component'] == 'Renamed catalog part')
        body = 'Renamed catalog body'
        host.command('body.rename', body=a['bodies'][0], name=body)
        host.command('appearance.assign', bodies=[body], color=[240, 10, 20])
        original = host.command('component.properties', component=a['id'])
        fresh = insert(thread_modeled=False)
        check(f'{label}: edited appearance is preserved and a clean definition is built',
              not fresh['component']['reused'] and fresh['library']['preserved_modified_definitions'][0]['component'] == a['id']
              and host.command('appearance.list')['bodies'][body]['color'] == '#f00a14')
        check(f'{label}: renamed component and body survive replacement unchanged',
              host.command('component.properties', component=a['id']) == original
              and definition('Renamed catalog part')['bodies'] == [body])
        replacement = insert(thread_modeled=False)
        check(f'{label}: replacement gets a free feature prefix and remains reusable',
              fresh['component']['component'] != a['name'] and replacement['component']['reused']
              and replacement['component']['component'] == fresh['component']['component'])
        check(f'{label}: unchanged detail remains reusable after its sibling was edited', insert(thread_modeled=True)['component']['reused'])
        if custom:
            changed = insert(thread_modeled=False, **custom)
            c = definition(changed['component']['component'])
            mc = metadata(c['id'])
            check(f'{label}: changed product has a distinct BOM number',
                  mc['org.techsquid.parts']['product'] != ra['product']
                  and mc['part_number'] != ma['part_number'] and mc['bom_type'] == 'normal')
        alias = {'techsquid-screws': 'M3x0.5', 'techsquid-nuts': 'M6x1'}.get(plugin)
        if alias:
            aliased = insert(size=alias, thread_modeled=True, **custom)
            alias_component = definition(aliased['component']['component'])
            alias_metadata = metadata(alias_component['id'])
            check(f'{label}: custom size spelling and detail preserve product and BOM number',
                  alias_metadata['org.techsquid.parts']['product'] == mc['org.techsquid.parts']['product']
                  and alias_metadata['part_number'] == mc['part_number'])
            rows = host.command('assembly.bom')['rows']
            purchase = next(row for row in rows if row['part_number'] == mc['part_number'])
            check(f'{label}: custom aliases share one purchase row across representations',
                  purchase['count'] == 2 and len(purchase['components']) == 2)
        if entry == 'make-nut':
            lock = {'style': 'Nylon-insert lock nut'}
            first_lock = insert(**lock, thread_modeled=False)
            second_lock = insert(**lock, thread_modeled=False)
            threaded_lock = insert(**lock, thread_modeled=True)
            lock_rows = [r for r in host.command('assembly.bom')['rows'] if r['part_number'] == 'DIN 985 M6']
            check('lock nuts: verified reuse and one purchase per two-body nut across thread details',
                  not first_lock['component']['reused'] and second_lock['component']['reused']
                  and not threaded_lock['component']['reused'] and len(lock_rows) == 1
                  and lock_rows[0]['count'] == 3 and len(lock_rows[0]['components']) == 2)
            extension = insert(**lock, size='M2.5', thread_modeled=False)
            supplier = metadata(definition(extension['component']['component'])['id'])
            check('lock nuts: the M2.5 extension carries its sourced supplier article',
                  extension['standard'] is None and extension['reference'] == 'Bossard BN 637'
                  and supplier['part_number'] == 'Bossard 1143735'
                  and json.loads(supplier['org.techsquid.parts']['product'])['kind'] == 'supplier_product')
        if entry == 'make-t-nut':
            mounted = insert(thread_modeled=False, slot='V-slot', along='Sketch Y')
            check(f'{label}: mounting orientation does not change the purchased nut', mounted['component']['reused'])
        check(f'{label}: insertion stays within the command budget',
              max(cosmetic['library']['host_calls'], modeled['library']['host_calls'], fresh['library']['host_calls']) < 248)


with Headless() as host:
    plugin = 'techsquid-extrusions'
    approval = host.command('plugin.inspect', path=package(plugin))['approval']
    for style, prefix in [('Pre-assembly', 'HNTT5'), ('Post-assembly', 'HNTP5')]:
        for size, suffix in [('M3', '3'), ('M4', '4'), ('M5', '5')]:
            host.command('plugin.run', path=package(plugin), approval=approval,
                         entry='make-t-nut', params={'type': style, 'size': size,
                                                     'thread_modeled': False})
    rows = host.command('assembly.bom')['rows']
    actual = {row['part_number']: row['count'] for row in rows}
    expected = {f'MISUMI {prefix}-{suffix}': 1 for prefix in ('HNTT5', 'HNTP5')
                for suffix in ('3', '4', '5')}
    check('slot nuts: six thread and style combinations keep separate purchase rows',
          actual == expected, f'| {actual}')

def renamed_definition_case(plugin, entry, params):
    # No sibling representation: its numbered body name would accidentally reserve
    # the original prefix and hide the feature-name collision after both renames.
    with Headless() as host:
        approval = host.command('plugin.inspect', path=package(plugin))['approval']

        def insert():
            result = host.command('plugin.run', path=package(plugin), approval=approval, entry=entry,
                                  params={**params, 'thread_modeled': False}, allow_error=True)
            return result.get('result', result)

        first = insert()
        original = next(c for c in host.command('component.list')['components']
                        if c['name'] == first['component']['component'])
        host.command('component.rename', component=original['id'], name='Edited part')
        for i, body in enumerate(original['bodies']):
            host.command('body.rename', body=body, name=f'Edited body {i}')
        host.command('appearance.assign', bodies=['Edited body 0'], color=[240, 10, 20])
        properties = host.command('component.properties', component=original['id'])
        fresh = insert()
        label = f'{plugin}/{entry}: isolated renamed definition'
        check(label + ' gets a clean replacement', 'error' not in fresh, fresh.get('error', ''))
        if 'error' in fresh:
            return
        check(label + ' preserves the edited original',
              host.command('component.properties', component=original['id']) == properties
              and host.command('appearance.list')['bodies']['Edited body 0']['color'] == '#f00a14'
              and fresh['library']['preserved_modified_definitions'][0]['component'] == original['id'])
        again = insert()
        check(label + ' reuses the replacement', not fresh['component']['reused']
              and again.get('component', {}).get('reused')
              and again['component']['component'] == fresh['component']['component'])


for plugin, entry, params, _ in CASES:
    renamed_definition_case(plugin, entry, params)

finish()
