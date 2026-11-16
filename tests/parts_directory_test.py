#!/usr/bin/env python3
"""The saved parts directory locates candidates; it never authorizes reuse."""
import copy
import json

from harness import check, finish, package
from headless import Headless

KEY = 'org.techsquid.parts.index'
RECORD = 'org.techsquid.parts'

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']

    def insert(size='625 2RS', allow_error=False):
        reply = host.command('plugin.run', path=path, approval=approval,
                             entry='make-bearing', params={'size':size}, allow_error=allow_error)
        return reply.get('result', reply)

    def directory():
        return host.command('document.inspect', measure=False)['metadata'][KEY]

    def write(value):
        host.command('document.metadata.set', key=KEY, value=value)

    def component(result):
        return next(c['id'] for c in host.command('component.list')['components']
                    if c['name'] == result['component']['component'])

    first = insert()
    original = component(first)
    index = directory()
    digest = next(iter(index['definitions']))
    index['definitions'][digest] = [99999999]
    write(index)
    missing = insert()
    check('missing candidate ID creates a clean definition', not missing['component']['reused']
          and component(missing) != original)
    check('missing candidate does not remove the original definition',
          any(c['id'] == original for c in host.command('component.list')['components']))

    current = component(missing)
    metadata = host.command('component.metadata', component=current)['metadata']
    record = copy.deepcopy(metadata[RECORD])
    representation = json.loads(record['representation'])
    representation['revision'] = 'obsolete'
    record['representation'] = json.dumps(representation, separators=(',', ':'), sort_keys=True)
    host.command('component.metadata.set', component=current, values={RECORD:record})
    revised = insert()
    check('obsolete representation is not adopted', not revised['component']['reused']
          and len(revised['library']['preserved_modified_definitions']) == 1)
    check('obsolete candidate metadata remains intact',
          host.command('component.metadata', component=current)['metadata'][RECORD] == record)

    current = component(revised)
    record = copy.deepcopy(host.command('component.metadata', component=current)['metadata'][RECORD])
    record['snapshot']['version'] = 1
    host.command('component.metadata.set', component=current, values={RECORD:record})
    migrated = insert()
    check('old snapshot version cannot authorize reuse', not migrated['component']['reused']
          and component(migrated) != current)
    check('old snapshot definition is preserved unchanged',
          host.command('component.metadata', component=current)['metadata'][RECORD] == record)

    shield = insert('105ZZ')
    shield_id = component(shield)
    shield_before = host.command('component.metadata', component=shield_id)
    index = directory()
    index['definitions'][digest] = [shield_id]
    write(index)
    mismatch = insert()
    check('digest candidate with a different full identity cannot be reused',
          not mismatch['component']['reused'] and component(mismatch) != shield_id
          and len(mismatch['library']['preserved_modified_definitions']) == 1)
    check('different product candidate is preserved unchanged',
          host.command('component.metadata', component=shield_id) == shield_before)

    valid = directory()
    for label, invalid in [('unsupported version', dict(valid, version=2)),
                           ('malformed directory', {'version':1, 'definitions':[]})]:
        write(invalid)
        before_doc = host.command('document.inspect', measure=False)
        before_components = host.command('component.list')
        error = insert(allow_error=True)
        check(label+' is refused without edits', 'error' in error
              and before_doc == host.command('document.inspect', measure=False)
              and before_components == host.command('component.list'))
        write(valid)

finish()
