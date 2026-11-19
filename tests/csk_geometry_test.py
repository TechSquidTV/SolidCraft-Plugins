#!/usr/bin/env python3
"""CSK exterior geometry against the independent manufacturer catalog dimensions."""
import math

from harness import ROOT, TMP, check, finish, package
from pathlib import Path
from headless import Headless


# Formsprag P-1352-FC p3, plus Yuki size 10; independent of the compiled catalog.
SIZES = [
    ('CSK8', 8, 22, 9),
    ('CSK10', 10, 30, 9),
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
# MCC-P-956-FC-EN-US 11/25 pp28–29, PEAK sizes 8/12PP, Yuki size 10,
# DKC 8P-2RS and PTI sealed PP. Depths are from the bore/OD apex.
KEYS = {8: (3, .5, 2, .5), 10: (3, .6, 2, .5), 12: (4, 1.8, 2, .6), 15: (5, 1.2, 2, .6),
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
        if bore == 10 and sealed or suffix == 'PP-2RS' and bore in (8,12):
            continue
        CASES.append((sku + suffix, bore, od, sealed_width if sealed else width, keyed, outer))

with Headless() as host:
    path = package('techsquid-bearings')
    info = host.command('plugin.inspect', path=path)
    approval = info['approval']
    entries = {entry['name']: entry['input_schema'] for entry in info['manifest']['runtime']['entries']}
    fields = entries['make-csk']['properties']
    titles = {opt['const']:opt['title'] for opt in fields['size']['oneOf']}
    check('one CSK clutch dialog lists all presets plus From scratch',
          'make-custom-csk' not in entries and titles.get('From scratch') == 'From scratch — Dust covers by default'
          and set(titles) == {'From scratch', *(case[0] for case in CASES)})
    for field in ('closure', 'inner_keyway', 'outer_keyway'):
        definition = fields[field]
        check(f'{field} labels the inherited default',
              definition['default'] == 'Preset'
              and {'const':'Preset', 'title':'Use starting preset'} in definition['oneOf'])
    for sku, bore, od, width, keyed, outer in CASES:
        result = host.command('plugin.run', path=path, approval=approval,
                              entry='make-csk', params={'size': sku})
        result = result.get('result', result)
        name = f'CSK clutch {sku} Outer ring'
        inner_name = f'CSK clutch {sku} Inner ring'
        bodies = {body['name']: body for body in
                  host.command('inspect.measure', summary=True)['bodies']}
        body = bodies[name]
        span = [body['bbox']['max'][axis] - body['bbox']['min'][axis]
                for axis in range(3)]
        check(f'{sku}: native mounting envelope', body['closed'] and
              all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(span, (od, od / 2 + math.sqrt((od / 2)**2 - (KEYS[bore][2] / 2)**2) if outer else od, width))))
        faces = host.command('model.faces', body=name)['faces'] + host.command('model.faces', body=inner_name)['faces']
        radii = sorted({face['cylinder']['radius'] for face in faces
                        if 'cylinder' in face})
        check(f'{sku}: catalog bore and OD', len(radii) == 4 and
              all(math.isclose(a, b, abs_tol=1e-6)
                  for a, b in zip((radii[0],radii[-1]), (bore / 2, od / 2))))
        if keyed:
            edges = host.command('model.edges', body=name)['edges'] + host.command('model.edges', body=inner_name)['edges']
            vertices = [p for edge in edges for p in (edge['start'], edge['end']) if p is not None]
            iw, depth, ow, outer_depth = KEYS[bore]
            if bore == 12 and outer:
                depth = 1.2
            slots = [(iw, bore / 2 + depth)]
            if outer:
                slots.append((ow, -(od / 2 - outer_depth)))
            for key_width, floor in slots:
                expected = [(x, floor, z) for x in (-key_width / 2, key_width / 2)
                            for z in (-width / 2, width / 2)]
                check(f'{sku}: through keyway width and floor {floor}', all(
                    any(all(math.isclose(a, b, abs_tol=1e-6) for a, b in zip(p, target))
                        for p in vertices) for target in expected))
        check(f'{sku}: two rings and two closure faces per component',
              sum(body_name.startswith(f'CSK clutch {sku} ') for body_name in bodies) == 4
              and not result['component']['reused'])
        appearance = host.command('appearance.list')['bodies']
        for side,sign in [('front',1),('back',-1)]:
            sealed = '2RS' in sku and (bore != 8 or sign > 0)
            closure_name = f"CSK clutch {sku} {'Seal' if sealed else 'Cover'} {side}"
            closure = bodies[closure_name]
            check(closure_name + ': flush annular face and appropriate appearance',
                not closure['closed'] and closure['volume_mm3'] is None
                and math.isclose(closure['bbox']['min'][2],sign*width/2,abs_tol=1e-6)
                and math.isclose(closure['bbox']['max'][2],sign*width/2,abs_tol=1e-6)
                and math.isclose(closure['area_mm2'],math.pi*(radii[2]**2-radii[1]**2),abs_tol=1e-5)
                and appearance[closure_name]['color'] == ('#404040' if sealed else '#a0a0a0'))
        # Independent exterior measurements from supplied STEP references.
        source_dimensions = {
            'CSK8':(5.72,9,.181985,.5,.363970,1),
            'CSK12-2RS':(9.105,13.295,.363970,1,.363970,1),
            'CSK15P':(11.055,15.245,.363970,1,.363970,1),
            'CSK15PP':(11.055,15.245,.363970,1,.363970,1),
            'CSK15-2RS':(11.055,15.245,.363970,1,.363970,1),
            'CSK40P':(28.515,33.495,1.273896,3.5,1.273896,3.5),
            'CSK40PP':(28.515,33.495,1.273896,3.5,1.273896,3.5),
            'CSK40-2RS':(28.515,33.495,1.273896,3.5,1.273896,3.5),
        }
        component = next(c for c in host.command('component.list')['components']
                         if c['name'] == result['component']['component'])
        spec = host.command('component.metadata',component=component['id'])['metadata']['org.techsquid.specification']
        check(sku + ': measured/estimated evidence is explicit',
              spec['exterior']['basis'] == ('measured' if sku in source_dimensions else 'estimated')
              and all(len(source['sha256']) == 64 for source in spec['exterior']['sources']))
        closure = 'Dust covers'
        if sku.endswith('-2RS'):
            closure = 'Single lip seal' if bore == 8 else 'Lip seals'
        check(sku + ': option title gives the inherited closure and nominal envelope',
              titles[sku] == f'{sku} — {closure} · {bore} × {od} × {width} mm')
        if sku in source_dimensions:
            ri,ro,ic,ia,oc,oa = source_dimensions[sku]
            check(sku + ': measured ring shoulders',
                all(math.isclose(a,b,abs_tol=1e-6) for a,b in zip(radii[1:3],(ri,ro))))
            for ring,edge,shoulder,radial,axial,direction in [(inner_name,bore/2,ri,ic,ia,1),(name,od/2,ro,oc,oa,-1)]:
                edges = host.command('model.edges',body=ring)['edges']
                section = [(math.hypot(p[0],p[1]),p[2]) for e in edges for p in (e['start'],e['end']) if p]
                expected = [(edge+direction*radial,sign*width/2) for sign in (-1,1)] + [(edge,sign*(width/2-axial)) for sign in (-1,1)]
                check(ring + ': measured chamfer section',all(any(
                    all(math.isclose(a,b,abs_tol=1e-5) for a,b in zip(p,target)) for p in section) for target in expected))
    rows = host.command('assembly.bom')['rows']
    check('CSK: 55 distinct unassigned specifications', len(rows) == len(CASES) and
          all(not row.get('part_number') and row['count'] == 1 for row in rows))

    for invalid in ['CSK50', 'invalid', 'CSK10-2RS']:
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
          spec['geometry']['inner_keyway'] == {'width': 4.0, 'depth': 1.2, 'angle_degrees': 0.0}
          and [spec['geometry'][k] for k in ('bore','diameter','width')] == [12,32,10]
          and spec['reference']['baseline']['inner_keyway']['depth'] == 1.8
          and spec['reference']['customized'] and 'Formsprag' in spec['reference']['source']['title']
          and 'purchasing' not in spec and 'purchasing_identity' not in spec
          and not record.get('part_number'))
    check('custom: reference vendor is absent from canonical design identity',
          'formsprag' not in record['org.techsquid.parts']['product'].lower())
    check('custom: identical overrides reuse; different geometry stays distinct',
          repeated['component']['reused'] and
          default['component']['component'] != custom['component']['component'])
    body = next(b for b in component['bodies'] if b.endswith('Inner ring'))
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
                   {'custom_bore':32}, {'custom_width':0}, {'closure':'invalid'},
                   {'size':'CSK12PP','inner_keyway_depth':7,'outer_keyway_depth':2.5}, {'outer_keyway':'Enabled'}, {'size':'From scratch'},
                   {'part_number': 'CL-12'}, {'purchase_supplier': 'Seller'}]:
        result = host.command('plugin.run', path=path, approval=approval, entry='make-csk',
                              params={'size': 'CSK12P', **params}, allow_error=True)
        check('custom: invalid keyway fails without edits', 'error' in result
              and host.command('document.inspect', measure=False) == before)

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']
    # All envelope dimensions changed, inner removed, outer added; then a
    # reference-free counterpart. Independently check cylindrical radii and slot vertices.
    for size in ('CSK8P', 'From scratch'):
        params = {'size':size, 'custom_bore':9, 'custom_diameter':25, 'custom_width':12,
                  'inner_keyway':'None', 'outer_keyway':'Enabled',
                  'outer_keyway_width':2, 'outer_keyway_depth':0.7}
        result = host.command('plugin.run', path=path, approval=approval, entry='make-csk', params=params)
        result = result.get('result', result)
        component = next(c for c in host.command('component.list')['components']
                         if c['name'] == result['component']['component'])
        body = next(b for b in component['bodies'] if b.endswith('Outer ring'))
        inner = next(b for b in component['bodies'] if b.endswith('Inner ring'))
        faces = host.command('model.faces', body=body)['faces'] + host.command('model.faces',body=inner)['faces']
        radii = sorted({f['cylinder']['radius'] for f in faces if 'cylinder' in f})
        check(size + ': custom bore and OD', len(radii) == 4 and all(math.isclose(a,b,abs_tol=1e-6) for a,b in zip((radii[0],radii[-1]),[4.5,12.5])), str(radii))
        edges = host.command('model.edges', body=body)['edges']
        vertices = [p for e in edges for p in (e['start'],e['end']) if p is not None]
        check(size + ': added outer slot follows custom envelope', all(
            any(all(math.isclose(a,b,abs_tol=1e-6) for a,b in zip(p,target)) for p in vertices)
            for target in [(-1,-11.8,-6),(1,-11.8,-6),(-1,-11.8,6),(1,-11.8,6)]))
        record = host.command('component.metadata', component=component['id'])['metadata']
        spec = record['org.techsquid.specification']
        check(size + ': removed inner keyway is recorded', spec['geometry']['inner_keyway'] is None)
        check(size + ': reference remains truthful',
              spec['reference'] is None if size == 'From scratch' else
              spec['reference']['baseline']['bore'] == 8 and spec['reference']['customized'])
        saved = str(Path(TMP) / ('csk-' + size + '.sc'))
        host.command('file.save_as', path=saved)
        host.command('doc.open', path=saved)
        again = host.command('plugin.run', path=path, approval=approval, entry='make-csk', params=params)
        again = again.get('result',again)
        check(size + ': full customization survives reopen and reuses', again['component']['reused']
              and host.command('component.metadata', component=component['id'])['metadata'] == record)

with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect',path=path)['approval']
    params = {'size':'CSK20P','inner_keyway_depth':7,'custom_width':0.5,'closure':'Lip seals'}
    result = host.command('plugin.run',path=path,approval=approval,entry='make-csk',params=params)
    result = result.get('result',result)
    component = next(c for c in host.command('component.list')['components'] if c['name'] == result['component']['component'])
    spec = host.command('component.metadata',component=component['id'])['metadata']['org.techsquid.specification']
    bodies = host.command('inspect.measure',summary=True)['bodies']
    check('thin custom: separate rings remain closed around a deep slot',
          len(bodies)==4 and sum(b['closed'] for b in bodies)==2)
    check('thin custom: adapted exterior records estimation and limited chamfer',
          spec['exterior']['basis']=='estimated' and spec['exterior']['adapted_for_keyways']
          and spec['exterior']['section']['inner_chamfer']['axial']==0.125)
    check('thin custom: closure override builds two seals without changing width',
          sum('Seal ' in b['name'] for b in bodies)==2 and spec['geometry']['width']==0.5
          and spec['geometry']['closure']=='lip_seals')
    again = host.command('plugin.run',path=path,approval=approval,entry='make-csk',params=params)
    again = again.get('result',again)
    check('thin custom: detailed geometry reuses unchanged',again['component']['reused'])

    rotated = host.command('plugin.run',path=path,approval=approval,entry='make-csk',
                           params={'size':'CSK15PP','inner_keyway_angle':90,'outer_keyway_angle':270})
    rotated = rotated.get('result',rotated)
    component = next(c for c in host.command('component.list')['components'] if c['name']==rotated['component']['component'])
    for label,x,half in [('Inner ring',-8.7,2.5),('Outer ring',16.9,1)]:
        body=next(b for b in component['bodies'] if b.endswith(label))
        edges=host.command('model.edges',body=body)['edges']
        vertices=[p for e in edges for p in (e['start'],e['end']) if p]
        check('clocking: '+label+' rotates independently',all(any(
            all(math.isclose(a,b,abs_tol=1e-6) for a,b in zip(p,target)) for p in vertices)
            for target in [(x,y,z) for y in (-half,half) for z in (-5.5,5.5)]))

def check_dimension_search(host, run):
    search = 'make-csk-by-dimensions'
    named = 'make-csk'
    before = host.command('document.inspect', measure=False)
    for filters, reasons in [
        ({}, ['enter at least one']),
        ({'match_bore':15, 'match_width':11}, ['CSK15 (', 'CSK15P (', 'CSK15PP (']),
        ({'match_bore':10, 'narrow_by_closure':'Lip seals'}, ['no CSK preset']),
    ]:
        error = run(search, filters, allow_error=True).get('error','')
        check(f'CSK search {filters} explains refusal before editing',
              all(reason in error for reason in reasons) and host.command('document.inspect', measure=False) == before, error)
    for entry, params in [
        (search, {'match_bore':0}),
        (search, {'match_bore':'15'}),
        (search, {'match_bore':15, 'match_outer_keyway':'Enabled'}),
        (search, {'match_bore':15, 'custom_width':12}),
        (named, {'size':'CSK15P', 'match_bore':15}),
        (named, {'size':'CSK15P', 'preset':'CSK15P'}),
        (named, {'size':'By dimensions'}),
        (named, {'size':'Custom'}),
        (named, {'size':'From scratch'}),
    ]:
        error = run(entry, params, allow_error=True).get('error','')
        check(f'{entry} rejects {params} without editing',
              bool(error) and host.command('document.inspect', measure=False) == before, error)

    for designation, filters, basis in [
        ('CSK15PP', {'match_bore':15, 'match_width':11, 'narrow_by_outer_keyway':'Enabled'}, 'measured'),
        ('CSK20P', {'match_diameter':47, 'narrow_by_closure':'Dust covers', 'narrow_by_inner_keyway':'Enabled', 'narrow_by_outer_keyway':'None'}, 'estimated'),
    ]:
        matched = run(search, filters)['component']
        name = 'CSK clutch '+designation
        check(designation+' is built by dimensions', matched['component'] == name)
        meta = host.command('component.metadata', component=name)['metadata']['org.techsquid.specification']
        exterior = meta['exterior']
        check(name+' records its baseline and exterior evidence',
              meta['geometry']['designation'] == designation and not meta['reference']['customized']
              and exterior['basis'] == basis and exterior['method'] and exterior['sources']
              and all(s['file'] and len(s['sha256']) == 64 for s in exterior['sources']))
        saved = str(Path(TMP)/'matched-csk.json')
        host.command('file.save_as', path=saved)
        host.command('doc.open', path=saved)
        direct = run(named, {'size':designation})['component']
        check(name+' reuses by designation after reopening',
              direct['reused'] and direct['component'] == name
              and host.command('component.metadata', component=name)['metadata']['org.techsquid.specification'] == meta)

    changed = run(named, {'size':'CSK15PP', 'custom_width':12})['component']
    spec = host.command('component.metadata', component=changed['component'])['metadata']['org.techsquid.specification']
    check('CSK clutch records the original baseline and estimates the edited exterior',
          spec['reference']['baseline']['width'] == 11 and spec['geometry']['width'] == 12
          and spec['geometry']['designation'] == 'CSK15PP' and spec['reference']['customized']
          and spec['exterior']['basis'] == 'estimated')

    filters = {'match_bore':15, 'match_width':11, 'narrow_by_inner_keyway':'None'}
    found = run(search, filters)['component']
    check('an absent inner keyway matches CSK15', found['component'] == 'CSK clutch CSK15')


with Headless() as host:
    path = package('techsquid-bearings')
    approval = host.command('plugin.inspect', path=path)['approval']

    def run_plugin(entry, params, allow_error=False):
        reply = host.command('plugin.run', path=path, approval=approval, entry=entry,
                             params=params, allow_error=allow_error)
        return reply.get('result', reply)

    retired = run_plugin('make-custom-csk', {'preset':'CSK12P'}, allow_error=True)
    check('the former Configure entry is rejected', 'unknown entry' in retired.get('error', ''), retired.get('error', ''))
    check_dimension_search(host, run_plugin)
    has_plugin_tools = any(c['id'] == 'plugin.tools' for c in host.tool('list_commands', {'filter':'plugin.tools'}))

# Hosts that publish installed plugins as tools also validate the generated input schema.
if has_plugin_tools:
    with Headless(plugins_dir=Path(ROOT)/'dist') as host:
        def run_tool(entry, params, allow_error=False):
            try:
                reply = host.tool(f'techsquid-bearings.{entry}', params, allow_error=allow_error)
            except RuntimeError as error:
                # Schema failures can arrive as JSON-RPC Invalid params before the guest runs.
                if allow_error and str(error).startswith("{'code': -32602,"):
                    return {'error': str(error)}
                raise
            return reply.get('result', reply)

        print('Checking CSK search through the generated tool')
        check_dimension_search(host, run_tool)

finish()
