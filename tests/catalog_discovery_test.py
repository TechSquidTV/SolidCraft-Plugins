#!/usr/bin/env python3
"""All packaged providers advertise the same inert discovery contract."""
from harness import check, finish, package
from headless import Headless

CATALOGS = {
    'techsquid-screws': (8, 'socket'),
    'techsquid-nuts': (29, 'm3'),
    'techsquid-inserts': (29, 'm3-5.7'),
    'techsquid-extrusions': (11, 'v-slot-2020'),
    'techsquid-bearings': (6, '30'),
    'techsquid-v-wheels': (5, 'solid-v-wheel'),
}

for plugin, (count, chosen) in CATALOGS.items():
    with Headless() as host:
        approval = host.command('plugin.inspect', path=package(plugin))['approval']
        host.command('component.create', name='Carrier', activate=True)
        sketch = host.command('sketch.create', name='Mount', plane='YZ')['sketch']
        host.command('sketch.circle.center', center=[12, 15], radius=1, id='mount')
        host.command('sketch.finish')
        # Retain a live selection, active component, undo stack and populated model.
        host.command('select.set', items=[{'type': 'sketch_curve', 'sketch': sketch, 'id': 'mount'}])

        def invoke(entry, params, allow_error=False):
            reply = host.command('plugin.run', path=package(plugin), approval=approval,
                                 entry=entry, params=params, allow_error=allow_error)
            return reply.get('result', reply)

        def state():
            return (host.command('document.inspect', measure=False),
                    host.command('component.list'), host.command('selection.get'))

        before = state()
        ids = []
        offset = 0
        while True:
            page = invoke('catalog', {'version': 1, 'request': {'operation': 'list', 'limit': 3, 'offset': offset}})
            ids.extend(item['id'] for item in page['result']['items'])
            offset = page['result']['next_offset']
            if offset is None:
                break
        check(f'{plugin}: bounded complete discovery', len(ids) == count and ids == sorted(set(ids)))
        revision = page['catalog']['revision']
        parts = []
        for part in ids:
            described = invoke('catalog', {'version': 1, 'request': {'operation': 'describe', 'part': part}})['result']['part']
            resolved = invoke('catalog', {'version': 1, 'revision': revision,
                                         'request': {'operation': 'resolve', 'part': part}})['result']['part']
            assert described['insertion'] == {key: resolved['insertion'][key] for key in ('plugin', 'entry')}
            assert resolved['description']['attachments']
            assert [m['product'] for m in resolved['members']] == resolved['representation']['product']['members']
            assert [m['representation'] for m in resolved['members']] == resolved['representation']['dependencies']
            parts.append(resolved)
        check(f'{plugin}: all items describe and resolve with named frames', len(parts) == count)
        check(f'{plugin}: discovery leaves the entire design context unchanged', state() == before)
        bad = invoke('catalog', {'version': 1, 'revision': 'obsolete',
                                'request': {'operation': 'resolve', 'part': chosen}}, allow_error=True)
        check(f'{plugin}: stale catalog is rejected without edits', 'error' in bad and state() == before)
        selected = parts[ids.index(chosen)]
        params = selected['insertion']['params']
        retired = {'techsquid-screws': 'designation', 'techsquid-nuts': 'custom_height',
                   'techsquid-inserts': 'modeled'}.get(plugin, 'typo_size')
        bad = invoke(selected['insertion']['entry'], {**params, retired: 'invalid'}, allow_error=True)
        check(f'{plugin}: retired or unknown fields fail without edits',
              'error' in bad and retired in bad['error'] and state() == before)
        params['catalog_selection']['catalog_revision'] = 'obsolete'
        bad = invoke(selected['insertion']['entry'], params, allow_error=True)
        check(f'{plugin}: insertion revalidates the selection before editing', 'error' in bad and state() == before)
        params['catalog_selection']['catalog_revision'] = revision
        inserted = invoke(selected['insertion']['entry'], params)
        check(f'{plugin}: advertised route inserts on the selected frame', inserted.get('normal') == [1, 0, 0])

finish()
