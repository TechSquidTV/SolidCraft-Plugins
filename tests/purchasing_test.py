#!/usr/bin/env python3
"""Native purchasing identity, BOM collision, annotation and rollback checks."""
from pathlib import Path
from harness import TMP, check, finish, package
from headless import Headless

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']

    def insert(**params):
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-csk', params={'size': 'CSK12P', **params})
        return result.get('result', result)['component']

    def metadata(component):
        return host.command('component.metadata', component=component['component'])['metadata']

    def reject(params, label):
        before = host.command('document.inspect', measure=False)
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-csk', params={'size': 'CSK12P', **params}, allow_error=True)
        check(label, 'error' in result and
              host.command('document.inspect', measure=False) == before, str(result)[:180])

    plain = insert()
    source = metadata(plain)['org.techsquid.specification']
    supplier = {'purchase_supplier': 'Seller A', 'purchase_sku': 'LISTING-12'}
    first = insert(**supplier, purchase_url='https://example.com/item/12')
    second = insert(**supplier, purchase_url='https://example.com/item/12?ref=other')
    meta = metadata(second)
    check('URL-only changes reuse one definition', second['reused'] and first['component'] == second['component'])
    check('purchase leaves geometry and drawing references untouched', meta['org.techsquid.specification'] == source)
    check('latest URL saved separately from identity',
          meta['org.techsquid.purchasing']['url'].endswith('?ref=other')
          and 'https://' not in meta['org.techsquid.parts']['product']
          and 'https://' not in meta['org.techsquid.purchasing']['purchasing_identity'])
    rows = host.command('assembly.bom')['rows']
    check('same supplier/SKU with changed URL has quantity two',
          len(rows) == 2 and sorted(r['count'] for r in rows) == [1, 2])
    host.command('edit.undo')
    check('undo restores previous URL and occurrence quantity together',
          metadata(first)['org.techsquid.purchasing']['url'] == 'https://example.com/item/12'
          and sorted(r['count'] for r in host.command('assembly.bom')['rows']) == [1, 1])
    host.command('edit.redo')
    check('redo restores the annotation and verified definition', metadata(first) == meta
          and sorted(r['count'] for r in host.command('assembly.bom')['rows']) == [1, 2])
    omitted = insert(**supplier)
    check('omitted URL preserves existing annotation and reuse', omitted['reused'] and metadata(omitted) == meta)
    explicit_number = insert(**supplier, part_number='CL-12', purchase_url='https://example.com/item/12')
    explicit_repeat = insert(**supplier, part_number='CL-12', purchase_url='https://example.com/item/12?tracking=yes')
    check('same BOM assignment reuses after URL update', explicit_repeat['reused'] and explicit_repeat['component'] == explicit_number['component'])
    for params, label in [
        ({'part_number':'CL-12','purchase_supplier':'Seller B','purchase_sku':'LISTING-12'}, 'same BOM number with another seller is rejected'),
        ({'part_number':'CL-12',**supplier,'inner_keyway_depth':1.2}, 'same BOM number with another keyway is rejected'),
        ({'part_number':'CL-12','purchase_supplier':'Seller A','purchase_sku':'OTHER'}, 'same BOM number with another SKU is rejected'),
        ({'part_number':'CL-12'}, 'same BOM number without the existing purchase is rejected'),
        ({'purchase_supplier':'Seller'}, 'supplier alone cannot identify a purchase'),
        ({'purchase_sku':'12'}, 'SKU alone cannot identify a purchase'),
        ({'purchase_url':'https://example.com'}, 'URL alone cannot identify a purchase'),
        ({**supplier,'purchase_manufacturer':'Brand'}, 'manufacturer requires a manufacturer part number'),
    ]:
        reject(params, label)
    rows = host.command('assembly.bom')['rows']
    check('conflicts cannot inflate or merge the existing BOM row',
          [r['count'] for r in rows if r['part_number'] == 'CL-12'] == [2])
    custom = insert(part_number='INTERNAL-12', inner_keyway_depth=1.2)
    check('internal number alone is a normal design with no purchase identity',
          metadata(custom)['bom_type'] == 'normal' and
          metadata(custom)['org.techsquid.purchasing']['purchasing_identity'] is None)
    alternate = insert(part_number='CL-12-B', purchase_supplier='Seller B', purchase_sku='12', inner_keyway_depth=1.2)
    check('a distinct number allows a distinct keyway and purchase', alternate['component'] != explicit_number['component'])
    branded = insert(purchase_supplier='Seller C', purchase_sku='12', purchase_manufacturer='Brand', purchase_mpn='P12')
    check('manufacturer product identifier is retained',
          metadata(branded)['org.techsquid.purchasing']['assignment']['purchase']['manufacturer'] == {'authority':'Brand','part_number':'P12'})

    # Collision with a non-CSK catalog product must be caught too.
    host.command('plugin.run', path=path, approval=approval, entry='make-bearing', params={'size':'625 2RS'})
    reject({'part_number':'OpenBuilds 30', **supplier}, 'existing radial product number cannot be reused for a CSK')

    saved = str(Path(TMP) / 'purchasing.sc')
    host.command('file.save_as', path=saved)
    host.command('doc.open', path=saved)
    reopened = insert(**supplier, part_number='CL-12')
    check('updated annotation and reuse survive reopening', reopened['reused'] and
          metadata(reopened)['org.techsquid.purchasing']['url'].endswith('?tracking=yes'))
    reject({'part_number':'CL-12','purchase_supplier':'Seller B','purchase_sku':'12'}, 'collision detection survives reopening')

    # Editing the native definition must not be blessed by a new URL annotation.
    components = host.command('component.list')['components']
    body = next(c['bodies'][0] for c in components if c['name'] == reopened['component'])
    host.command('appearance.assign', bodies=[body], color=[220, 40, 40])
    reject({'part_number':'CL-12', **supplier, 'purchase_url':'https://example.com/new'}, 'modified numbered definition rejects insertion without edits')

    unnumbered_body = next(c['bodies'][0] for c in components if c['name'] == first['component'])
    host.command('appearance.assign', bodies=[unnumbered_body], color=[40, 220, 40])
    old_metadata = metadata(first)
    replacement = insert(**supplier, purchase_url='https://example.com/new')
    check('URL update cannot bless modified unnumbered geometry', not replacement['reused']
          and replacement['component'] != first['component'] and metadata(first) == old_metadata)
    repeat_replacement = insert(**supplier)
    check('new clean definition remains reusable', repeat_replacement['reused']
          and repeat_replacement['component'] == replacement['component'])

finish()
