#!/usr/bin/env python3
"""Run the compiled sketch-driven extrusion plugin through the real host.

Run after ./build.sh. SOLIDCRAFT_CLI may select an unsigned host build.
"""
import math
import os
from harness import check, finish, inspect, package, run, killed, skip, TMP

PLUGIN = 'techsquid-extrusions'

def cmd(command, **params):
    return dict(command=command, params=params)

def path_steps():
    steps = [cmd('parameters.add', name=n, expression=f'{v} mm')
             for n,v in [('X',20),('Z',20),('L',100),('L2',140),('DX',0)]]
    # An arbitrary non-world-aligned sketch plane exercises component-free 3D placement.
    steps += [cmd('sketch.create', name='Paths', plane={
        'x_dir':[0,1,0], 'y_dir':[0,0,1], 'origin':[30,0,0]})]
    for i,x,length in [(1,20,100),(2,100,140)]:
        steps += [cmd('sketch.line',points=[[x,20],[x,20+length]])]
        base = 'X' if i == 1 else 'X + 80 mm'
        for point,axis,value in [(f'l{i}.start','horizontal',base),
                                 (f'l{i}.start','vertical','Z'),
                                 (f'l{i}.end','horizontal',base+' + DX'),
                                 (f'l{i}.end','vertical','Z + '+('L' if i==1 else 'L2'))]:
            steps += [cmd('sketch.dimension',entities=['origin',point],type=axis,value=value)]
    return steps + [cmd('sketch.finish')]

def plugin(slot,size,**options):
    return cmd('plugin.run',path=package(PLUGIN),approval=APPROVAL,entry='make-extrusion',params={
        'mode':'Sketch driven','sketch':'Paths','slot':slot,'size':size,**options})

code,text,meta=inspect(PLUGIN)
if code:
    check('inspect',False,text[-1000:]); finish()
APPROVAL=meta['approval']

for slot,sizes in [('T-slot',['2020','2040','2060','2080','4040']),('V-slot',['2020','2040','2060','2080'])]:
    for size in sizes:
        label=f'{slot} {size}'
        save=os.path.join(TMP,'rails.sc')
        code,text,results=run(path_steps()+[plugin(slot,size),cmd('inspect.measure',summary=True)],save=save)
        if killed(code): skip(label,'host killed loading plugin'); continue
        check(label+' builds two rails',code==0,text[-600:] if code else '')
        if code: continue
        made=results[-2]; area=made['area_mm2']; measured=results[-1]
        check(label+' independent lengths',made['count']==2 and len(measured['bodies'])==2 and
              all(b['closed'] and abs(b['volume_mm3']-area*length)<area*length*.002
                  for b,length in zip(measured['bodies'],[100,140])))
        edits=[cmd('parameters.change',name=n,expression=v) for n,v in
               [('X',60),('Z',40),('DX',-40),('L',150),('L2',180)]]
        code,text,values=run(edits+[cmd('inspect.measure',summary=True),cmd('document.inspect',measure=False,sketches=False)],inp=save,save=save)
        check(label+' edits',code==0,text[-600:] if code else '')
        if code: continue
        check(label+' volumes follow both paths',all(b['closed'] and abs(b['volume_mm3']-area*math.hypot(length,40))<area*length*.002
              for b,length in zip(values[-2]['bodies'],[150,180])))
        check(label+' healthy timeline',all(not f.get('error') for f in values[-1]['timeline']))
        check(label+' fixed sections',all(s['dof']==0 for s in values[-1]['sketches'] if s['name'].endswith(': Section')))
        code,text,reopened=run([cmd('inspect.measure',summary=True)],inp=save)
        check(label+' reopens',code==0 and reopened[-1]['body_count']==2,text[-600:] if code else '')

# Fully modeled off-axis bores: two 2040 rails, one end each = four taps.
for slot,size,taps,modeled in [('T-slot','2040','End',True),('T-slot','4040','Both',False),('V-slot','2080','Both',False)]:
    label=f'machined {slot} {size}'
    save=os.path.join(TMP,'machined.sc')
    code,text,res=run(path_steps()+[plugin(slot,size,end_taps=taps,thread_modeled=modeled,
        holes='+X 25 -Y 40',holes_from='End'),cmd('inspect.measure',summary=True)],save=save)
    if killed(code): skip(label,'host killed loading plugin'); continue
    check(label+' builds',code==0,text[-700:] if code else '')
    if code: continue
    names=res[-2]['bodies']
    code,text,forward=run([cmd('parameters.change',name=n,expression=v) for n,v in [('X',40),('Z',30),('DX',30),('L',120),('L2',160)]]+[cmd('document.inspect',measure=False,sketches=False)],inp=save,save=save)
    check(label+' combined forward tilt',code==0 and all(not f.get('error') for f in forward[-1]['timeline']),text[-600:] if code else '')
    if code: continue
    edits=[cmd('parameters.change',name=n,expression=v) for n,v in [('X',60),('Z',40),('DX',-40),('L',150),('L2',180)]]
    code,text,res=run(edits+[cmd('model.faces',body=n) for n in names]+[cmd('document.inspect',measure=False,sketches=False)],inp=save,save=save)
    check(label+' reverse tilt and translation',code==0,text[-700:] if code else '')
    if code: continue
    check(label+' healthy after edits',all(not f.get('error') and not f.get('warning') for f in res[-1]['timeline']))
    for index,faces in enumerate(res[len(edits):-1]):
        rise=[150,180][index]; length=math.hypot(40,rise); d=[0,-40/length,rise/length]; start=[30,60+80*index,40]
        cylinders=[f for f in faces['faces'] if f.get('cylinder')]
        clearance=[f for f in cylinders if abs(f['cylinder']['radius']-2.75)<1e-5]
        along={round(sum((f['cylinder']['origin'][j]-start[j])*d[j] for j in range(3)),3) for f in clearance}
        check(label+f' rail {index+1} holes follow end offsets',all(any(abs(a-(length-offset))<.01 for a in along) for offset in [25,40]))
        check(label+f' rail {index+1} holes perpendicular',bool(clearance) and all(abs(sum(f['cylinder']['axis'][j]*d[j] for j in range(3)))<1e-5 for f in clearance))
        if modeled:
            roots=[f for f in cylinders if abs(f['cylinder']['radius']-2.5)<1e-5]
            at=[sum((f['centroid'][j]-start[j])*d[j] for j in range(3)) for f in roots]
            check(label+f' rail {index+1} modeled end taps',len(roots)>=2 and all(length-12.01<=x<=length+.01 for x in at))
    code,text,reopened=run([cmd('inspect.measure',summary=True)],inp=save)
    check(label+' reopens',code==0 and all(b['closed'] for b in reopened[-1]['bodies']),text[-500:] if code else '')

# Selection mode and explicit subsets must not accidentally generate every line.
for select in [False, True]:
    steps=path_steps()
    options={'path_lines':'l2'}
    if select:
        steps += [cmd('select.set',items=[{'type':'sketch_curve','sketch':1,'id':'l2'}])]
        options={'sketch':''}
    code,text,res=run(steps+[plugin('T-slot','2020',**options),cmd('inspect.measure',summary=True)])
    check(('selected' if select else 'explicit')+' subset',code==0 and res[-2]['lines']==['l2'] and res[-1]['body_count']==1,text[-600:] if code else '')

# Repeated runs use unique body names and preserve their own dependencies.
code,text,res=run(path_steps()+[plugin('T-slot','2020',path_lines='l1'),plugin('V-slot','2020',path_lines='l2'),plugin('T-slot','2020',path_lines='l1'),cmd('inspect.measure',summary=True)])
check('mixed profiles and repeated runs',code==0 and res[-1]['body_count']==3 and len({b['name'] for b in res[-1]['bodies']})==3,text[-600:] if code else '')

# Regression for the prototype's almost-vertical boolean failure.
code,text,res=run(path_steps()+[plugin('T-slot','2020',end_taps='Both',thread_modeled=False,holes='+X 25'),cmd('parameters.change',name='DX',expression='0.00001 mm'),cmd('document.inspect',measure=False,sketches=False)])
check('near-vertical machining',code==0 and all(not f.get('error') for f in res[-1]['timeline']),text[-600:] if code else '')

steps=[cmd('sketch.create',name='Paths',plane='XZ')]
steps += [cmd('sketch.line',points=[[i*60,0],[i*60,100]]) for i in range(16)]
steps += [cmd('sketch.constraint.fix',entities=[f'l{i}' for i in range(1,17)],fixed=True),cmd('sketch.finish')]
code,text,res=run(steps+[plugin('T-slot','4040'),cmd('inspect.measure',summary=True)])
check('sixteen plain 4040 rails fit one run',code==0 and res[-2]['count']==16 and res[-1]['body_count']==16 and all(b['closed'] for b in res[-1]['bodies']),text[-600:] if code else '')

finish()
