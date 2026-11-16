#!/usr/bin/env python3
"""Compiled plugin integration: identity, composition, edits, undo and rollback."""
import json
import math
import os
from pathlib import Path
import shutil

from harness import TMP, check, finish, package
from headless import Headless

BEARINGS = 'techsquid-bearings'
WHEELS = 'techsquid-v-wheels'


def definition(listed, name):
    return next(c for c in listed['components'] if c['name'] == name)


def without_directory_write(plugin):
    """A real package that fails late, without changing its production manifest."""
    denied = Path(TMP)/'denied'/plugin
    if not denied.exists():
        shutil.copytree(package(plugin), denied)
        manifest = json.loads((denied/'plugin.json').read_text())
        manifest['runtime']['commands'].remove('document.metadata.set')
        (denied/'plugin.json').write_text(json.dumps(manifest))
    return str(denied)


def same_point(actual, expected):
    return len(actual) == len(expected) and all(
        math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-7) for a, b in zip(actual, expected))


def activation_case(plugin, nested):
    """Keep authoring through the same exact copy across whole insertions."""
    label = plugin+(': nested occurrence' if nested else ': copied occurrence')
    with Headless() as host:
        parent = host.command('component.create', name='Assembly', activate=True, grounded=False) if nested else None
        made = host.command('component.create', name='Carrier', parent=parent['component'] if parent else 0,
                            activate=True, grounded=False)
        carrier = made['component']
        sketch = host.command('sketch.create', name='Mount', plane='XY')['sketch']
        host.command('sketch.circle.center', center=[10,20], radius=1, id='mount')
        host.command('sketch.finish')
        if nested:
            host.command('occurrence.move', occurrence=made['occurrence'], translate=[5,0,0])
            copied = host.command('occurrence.copy', component=parent['component'], parent=0)['occurrence']
            # Rotate the copied parent 90 degrees around Y, then translate it.
            # The carrier occurrence ID is shared by the two complete paths.
            host.command('occurrence.move', occurrence=copied,
                         to_matrix=[[0,0,1,100],[0,1,0,50],[-1,0,0,30]])
            path = [copied, made['occurrence']]
            point, normal = [100,70,15], [1,0,0]
            axes = [[0,0,-1,0],[0,1,0,0],[1,0,0,0]]
        else:
            copied = host.command('occurrence.copy', component=carrier, parent=0, translate=[100,0,0])['occurrence']
            path = [copied]
            point, normal = [110,20,0], [0,0,1]
            axes = [[1,0,0,0],[0,1,0,0],[0,0,1,0]]
        host.command('component.activate', component=carrier, occurrence=path[-1], path=path)

        def mount_world():
            return next(p['world'] for p in host.command('sketch.inspect', sketch=sketch)['points']
                        if p['id'] == 'mount.center')

        entry = 'make-bearing' if plugin == BEARINGS else 'make-v-wheel'
        params = {'sketch':'Mount', 'sketch_point':'mount.center'}
        if plugin == BEARINGS:
            params['size'] = 'B626'
        check(label+': fixture selects the intended world point', same_point(mount_world(), point))
        # Failure after creation must restore the selected path, not only its
        # component ID. This uses the same late-failure fixture as the main suite.
        denied = without_directory_write(plugin)
        denied_approval = host.command('plugin.inspect', path=denied)['approval']
        before_doc = host.command('document.inspect', measure=False)
        before_components = host.command('component.list')
        failed = host.command('plugin.run', allow_error=True, path=denied, approval=denied_approval,
                              entry=entry, params=params)
        check(label+': late failure reaches the directory write', 'document.metadata.set' in str(failed.get('error')))
        check(label+': failed insertion preserves design and activation context',
              host.command('document.inspect', measure=False) == before_doc
              and host.command('component.list') == before_components and same_point(mount_world(), point))

        approval = host.command('plugin.inspect', path=package(plugin))['approval']
        for reused in (False, True):
            stage = label+(': reused definition' if reused else ': fresh definition')
            result = host.command('plugin.run', path=package(plugin), approval=approval, entry=entry, params=params)
            result = result.get('result', result)
            listed = host.command('component.list')
            part = definition(listed, result['component']['component'])
            placed = next(o for o in part['occurrences'] if o['id'] == result['component']['occurrences'][0])
            folder = next(c for c in listed['components'] if c['id'] == placed['parent'])
            check(stage+': expected creation or reuse', result['component']['reused'] == reused)
            check(stage+': insertion resolves the selected copy', same_point(result['at'], point)
                  and same_point(result['normal'], normal))
            check(stage+': native occurrence has the expected world frame',
                  len(folder['occurrences']) == 1 and folder['occurrences'][0]['parent'] == 0
                  and folder['occurrences'][0]['transform'] == [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]
                  and all(same_point(a,b) for a,b in zip(placed['transform'], axes+[point+[1]])))
            check(stage+': original authoring context is preserved', listed['active'] == carrier
                  and listed['active_path'] == path and same_point(mount_world(), point))


with Headless() as host:
    approvals = {p:host.command('plugin.inspect',path=package(p))['approval'] for p in [BEARINGS,WHEELS]}

    def insert(plugin, **params):
        result = host.command('plugin.run',path=package(plugin),approval=approvals[plugin],
                              entry='make-bearing' if plugin == BEARINGS else 'make-v-wheel',params=params)
        return result.get('result',result)

    carrier = host.command('component.create',name='Carrier',activate=True,grounded=False)['component']
    host.command('sketch.create',name='Mounts',plane={'origin':[0,0,0],'x_dir':[0,1,0],'y_dir':[0,0,1]})
    host.command('sketch.circle.center',center=[40,20],radius=1,id='a')
    host.command('sketch.circle.center',center=[70,20],radius=1,id='b')
    host.command('sketch.finish')
    before = host.command('document.inspect',measure=False)
    direct = insert(BEARINGS)
    wheel = insert(WHEELS,sketch='Mounts',sketch_point='a.center')
    repeated = insert(WHEELS,sketch='Mounts',sketch_point='b.center')
    another = insert(BEARINGS)
    listed = host.command('component.list')
    after = host.command('document.inspect',measure=False)
    bearing = definition(listed,'Bearing 625 2RS')
    assembly = definition(listed,'Solid V Wheel assembly')
    check('direct and assembled bearings share one definition', len([c for c in listed['components'] if c['name'].startswith('Bearing ')]) == 1
          and len(bearing['bodies']) == 4)
    check('subsequent direct and wheel insertions reuse geometry', another['component']['reused'] and repeated['component']['reused'])
    check('assembly contains one wheel, two same-bearing occurrences, one shim', len(assembly['children']) == 4
          and sum(c['component'] == bearing['id'] for c in assembly['children']) == 2)
    check('bearing child positions match the sourced stack', sorted(c['transform'][3][2] for c in assembly['children'] if c['component'] == bearing['id']) == [-3,3])
    check('each insertion adds exactly one undo step',after['undo'] == before['undo']+4)
    check('active component is restored',listed['active'] == carrier)
    check('arbitrary sketch plane controls the wheel axle',wheel['normal'] == [1,0,0] and wheel['at'] == [0,40,20]
          and repeated['at'] == [0,70,20])
    bom = host.command('assembly.bom')
    check('BOM counts purchases, not closure bodies or assembly kits', {r['part_number']:r['count'] for r in bom['rows'] if r['part_number']}
          == {'OpenBuilds 30':6,'OpenBuilds 05':2,'Solid V Wheel':2})
    check('repeat assembly has comfortable host-call headroom',repeated['library']['host_calls'] < 64,str(repeated['library']['host_calls']))
    # Undo/redo affects the whole insertion, not one of its internal commands.
    host.command('edit.undo')
    check('one undo removes the last bearing occurrence', len(definition(host.command('component.list'),'Bearing 625 2RS')['occurrences']) == len(bearing['occurrences'])-1)
    host.command('edit.redo')
    check('one redo restores the complete insertion',host.command('component.list') == listed)

    saved = os.path.join(TMP,'library.sc')
    host.command('file.save_as',path=saved)
    host.command('doc.open',path=saved)
    reopened = insert(WHEELS,sketch='Mounts',sketch_point='a.center')
    check('shared assembly remains reusable after save/reopen',reopened['component']['reused'])
    host.command('component.rename',component=bearing['id'],name='My bearing')
    renamed = insert(BEARINGS)
    check('display renaming preserves catalog reuse',renamed['component']['reused'] and renamed['component']['component'] == 'My bearing')

    host.command('appearance.assign',bodies=['Bearing 625 2RS Seal front'],color=[255,0,0])
    changed = insert(BEARINGS)
    check('changed bearing is preserved and a clean definition is built',not changed['component']['reused']
          and len(changed['library']['preserved_modified_definitions']) == 1)
    check('fresh multi-body part leaves the original seal appearance untouched',
          host.command('appearance.list')['bodies']['Bearing 625 2RS Seal front']['color'] == '#ff0000')
    new_list = host.command('component.list')
    fresh = definition(new_list,changed['component']['component'])
    check('original assembly still references the edited original',all(c['component'] != fresh['id'] for c in definition(new_list,'Solid V Wheel assembly')['children']))
    new_assembly = insert(WHEELS,sketch='Mounts',sketch_point='a.center')
    check('edited child prevents reuse of the old assembly',not new_assembly['component']['reused']
          and len(new_assembly['library']['preserved_modified_definitions']) == 1)
    new_list = host.command('component.list')
    rebuilt = definition(new_list,new_assembly['component']['component'])
    check('fresh assembly uses two occurrences of the fresh bearing',sum(c['component'] == fresh['id'] for c in rebuilt['children']) == 2)
    check('fresh assembly reuses unchanged wheel and shim definitions',len([c for c in new_list['components'] if c['name'] == 'Solid V Wheel']) == 1
          and len([c for c in new_list['components'] if c['name'].startswith('Precision shim')]) == 1)

    # Fail at the directory write, after an entire new 105ZZ has been created.
    # Removing one permitted command is a test fixture, not a production fallback.
    denied = without_directory_write(BEARINGS)
    approval = host.command('plugin.inspect',path=denied)['approval']
    before_doc = host.command('document.inspect',measure=False)
    before_components = host.command('component.list')
    failed = host.command('plugin.run',allow_error=True,path=denied,approval=approval,
                          entry='make-bearing',params={'size':'105ZZ'})
    check('fixture fails at the final directory write','document.metadata.set' in str(failed.get('error')))
    check('failed insertion rolls back geometry, metadata, undo and active context',host.command('document.inspect',measure=False) == before_doc
          and host.command('component.list') == before_components)

    mini = insert(WHEELS,size='Delrin Mini V Wheel')
    mini_list = host.command('component.list')
    mini_assembly = definition(mini_list,mini['component']['component'])
    mini_bearing = definition(mini_list,'Bearing 105ZZ')
    check('mini assembly uses corrected bearing stack',sorted(c['transform'][3][2] for c in mini_assembly['children'] if c['component'] == mini_bearing['id']) == [-2.5,2.5])
    check('mini assembly uses its own precision shim',any(c['name'] == 'Precision shim 8 x 5 x 1' for c in mini_list['components']))
    check('bearing first built by wheels is reused by Bearings',insert(BEARINGS,size='105ZZ')['component']['reused'])

    host.command('sketch.create',name='Batch points',plane='XY')
    for i in range(32):
        host.command('sketch.circle.center',center=[30*i,100],radius=1,id=f'p{i}')
    host.command('sketch.finish')
    batch = {'size':'Xtreme Mini V Wheel','sketch':'Batch points','sketch_point':','.join(f'p{i}.center' for i in range(32))}
    before_doc = host.command('document.inspect',measure=False)
    refused = host.command('plugin.run',allow_error=True,path=package(WHEELS),approval=approvals[WHEELS],entry='make-v-wheel',params=batch)
    check('oversized creation batch is refused by preflight','creation batch exceeds' in str(refused.get('error')))
    check('preflight refusal preserves the complete design',host.command('document.inspect',measure=False) == before_doc)
    batch['sketch_point'] = ','.join(f'p{i}.center' for i in range(24))
    placed = insert(WHEELS,**batch)
    check('large accepted batch creates one assembly definition',placed['count'] == 24 and not placed['component']['reused']
          and placed['library']['host_calls'] < 248,str(placed['library']['host_calls']))
    batch['sketch_point'] = ','.join(f'p{i}.center' for i in range(32))
    placed = insert(WHEELS,**batch)
    check('existing assembly allows the full 32-placement batch',placed['count'] == 32 and placed['component']['reused']
          and placed['library']['host_calls'] < 160,str(placed['library']['host_calls']))

    # A folder is itself a shared definition. Adding a child to a copied folder
    # would add it to every copy, even if an explicit parent path removed ambiguity.
    folder = definition(host.command('component.list'), 'Bearings')
    for label, parent in [('root copy', 0), ('nested copy', carrier)]:
        host.command('occurrence.copy', component=folder['id'], parent=parent, translate=[100,0,0])
        host.command('component.activate', component=carrier)
        before_components = host.command('component.list')
        copied_folder = definition(before_components, folder['name'])
        before_bom = {r['part_number']:r['count'] for r in host.command('assembly.bom')['rows']}
        result = host.command('plugin.run', allow_error=True, path=package(BEARINGS),
                              approval=approvals[BEARINGS], entry='make-bearing', params={})
        check(label+': insertion into copied-folder design succeeds', 'error' not in result, str(result.get('error','')))
        if 'error' in result:
            continue
        result = result.get('result', result)
        after_components = host.command('component.list')
        placed_bearing = definition(after_components, result['component']['component'])
        occurrence = next(o for o in placed_bearing['occurrences'] if o['id'] == result['component']['occurrences'][0])
        folder = next(c for c in after_components['components'] if c['id'] == occurrence['parent'])
        check(label+': existing bearing definition is reused', result['component']['reused'])
        check(label+': copied folder and all of its children remain unchanged',
              definition(after_components, copied_folder['name']) == copied_folder)
        check(label+': new folder has one root placement and is phantom', folder['id'] != copied_folder['id']
              and [o['parent'] for o in folder['occurrences']] == [0]
              and host.command('component.metadata', component=folder['id'])['bom_type'] == 'phantom')
        before_bom['OpenBuilds 30'] += 1
        check(label+': exactly one purchase is added',
              {r['part_number']:r['count'] for r in host.command('assembly.bom')['rows']} == before_bom)
        check(label+': active component is restored', after_components['active'] == carrier)
        host.command('edit.undo')
        check(label+': one undo removes the whole insertion', host.command('component.list') == before_components)
        host.command('edit.redo')
        check(label+': redo restores the complete insertion', host.command('component.list') == after_components)
        host.command('file.save_as', path=saved)
        host.command('doc.open', path=saved)
        again = insert(BEARINGS)
        listed = host.command('component.list')
        placed_bearing = definition(listed, again['component']['component'])
        occurrence = next(o for o in placed_bearing['occurrences'] if o['id'] == again['component']['occurrences'][0])
        check(label+': later insertions reuse the numbered folder after reopening', occurrence['parent'] == folder['id']
              and len(listed['components']) == len(after_components['components']))
        before_bom['OpenBuilds 30'] += 1
        check(label+': repeated insertion still adds only one purchase',
              {r['part_number']:r['count'] for r in host.command('assembly.bom')['rows']} == before_bom)

for plugin in (BEARINGS, WHEELS):
    for nested in (False, True):
        activation_case(plugin, nested)

finish()
