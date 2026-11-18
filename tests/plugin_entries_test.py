#!/usr/bin/env python3
"""Packages expose insertion workflows only; removed discovery cannot edit a design."""
import json
from pathlib import Path
import shutil

from harness import TMP, check, finish, package, picks, plugin_ids
from headless import Headless

for plugin in plugin_ids():
    with Headless() as host:
        info = host.command('plugin.inspect', path=package(plugin))
        approval = info['approval']
        entries = info['manifest']['runtime']['entries']
        check(f'{plugin}: only insertion entries are advertised', bool(entries)
              and all(e['effect'] == 'edit' and e['name'] != 'catalog' for e in entries))
        host.command('component.create', name='Carrier', activate=True)
        sketch = host.command('sketch.create', name='Mount', plane='YZ')['sketch']
        host.command('sketch.circle.center', center=[12, 15], radius=1, id='mount')
        host.command('sketch.finish')

        def state():
            return (host.command('document.inspect', measure=False),
                    host.command('component.list'), host.command('selection.get'))

        before = state()
        query = {'version': 1, 'request': {'operation': 'list'}}
        bad = host.command('plugin.run', path=package(plugin), approval=approval,
                           entry='catalog', params=query, allow_error=True)
        check(f'{plugin}: removed catalog entry is rejected without edits',
              'unknown entry' in bad.get('error', '') and state() == before, bad.get('error', ''))

        # A temporary declaration must not resurrect the old compiled dispatch.
        # This exercises the guest, beyond the real package's manifest refusal.
        redeclared = Path(TMP)/plugin
        shutil.copytree(package(plugin), redeclared)
        manifest_path = redeclared/'plugin.json'
        manifest = json.loads(manifest_path.read_text())
        manifest['runtime']['entries'].append({
            'name': 'catalog', 'effect': 'read', 'input_schema': {'type': 'object'},
        })
        manifest_path.write_text(json.dumps(manifest))
        altered = host.command('plugin.inspect', path=str(redeclared))['approval']
        bad = host.command('plugin.run', path=str(redeclared), approval=altered,
                           entry='catalog', params=query, allow_error=True)
        check(f'{plugin}: no dormant catalog handler remains in the guest',
              'unknown entry catalog' in bad.get('error', '') and state() == before)

        entry = entries[0]
        properties = entry['input_schema']['properties']
        params = {key: value['default'] for key, value in properties.items() if 'default' in value}
        params['placement'] = picks(sketch, 'mount', curve=True)
        if 'create' in properties:
            # Insertion-only choice comes from the public form, not another part table.
            params['create'] = next(c for c in properties['create']['enum']
                                    if c.endswith(' only') and c not in ('Hole only', 'Pocket only'))
        if 'thread_modeled' in properties:
            params['thread_modeled'] = False
        retired = {'techsquid-screws': 'designation', 'techsquid-nuts': 'custom_height',
                   'techsquid-inserts': 'modeled'}.get(plugin, 'typo_size')
        bad = host.command('plugin.run', path=package(plugin), approval=approval, entry=entry['name'],
                           params={**params, retired: 'invalid'}, allow_error=True)
        # SolidCraft 0.44 passes the plugin's "<key>: unsupported field" through. Newer hosts
        # swap a key for its title and drop a key the form does not have, leaving the reason.
        error = bad.get('error', '')
        check(f'{plugin}: retired or unknown fields still fail without edits',
              (retired in error or 'unsupported field' in error) and state() == before,
              error[-300:])
        inserted = host.command('plugin.run', path=package(plugin), approval=approval,
                                entry=entry['name'], params=params)
        inserted = inserted.get('result', inserted)
        check(f'{plugin}: insertion uses the picked non-XY frame',
              inserted.get('normal') == [1, 0, 0])

finish()
