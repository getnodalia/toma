#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Original silent-cinema galop for the sample film, synthesised from scratch. Run in the repo root.
import numpy as np, random, subprocess
SR=32000; BPM=132; Q=60/BPM; BAR=2*Q
rng=random.Random(1902)
def hz(m): return 440*2**((m-69)/12)
def note(m,dur,vel=0.5):
    n=int((dur+0.35)*SR); t=np.arange(n)/SR; f=hz(m); y=np.zeros(n)
    for k,a in enumerate([1,.55,.38,.22,.14,.09,.06],1):
        y+=a*np.sin(2*np.pi*f*k*t*(1+0.0004*k*k))*np.exp(-t*(2.2+k*1.3)/(1+max(0,(m-60))/40))
    env=np.minimum(t/0.004,1)*np.exp(-np.maximum(t-dur,0)*14)
    return y*env*vel
CH={'Am':(57,[69,72,76],'A'),'E7':(52,[68,71,74],'E'),'C':(48,[67,72,76],'C'),'G7':(43,[65,71,74],'G'),'F':(53,[69,72,77],'F'),'Dm':(50,[69,72,77],'D')}
SC={'Am':[69,71,72,74,76,77,80,81],'E7':[68,69,71,72,74,76,77,80],'C':[67,69,71,72,74,76,77,79],'G7':[67,69,71,74,76,77,79],'F':[69,70,72,74,76,77,79,81],'Dm':[69,70,72,74,76,77,79,81]}
sections={'A':['Am','Am','E7','Am','Am','Dm','E7','Am'],
          'B':['C','C','G7','C','F','C','G7','C'],
          'C':['Am','E7','Am','E7','Dm','Am','E7','Am']}
plan=('A A B B A C B A C A B B A C B A C A ' * 1).split()
total_bars=sum(8 for _ in plan)+2
total=total_bars*BAR
out=np.zeros(int((total+2)*SR))
def put(y,t):
    i=int(t*SR); out[i:i+len(y)]+=y[:len(out)-i]
bar=0; prev=76
for si,s in enumerate(plan):
    for ci,c in enumerate(sections[s]):
        root,chord,_=CH[c]; t0=bar*BAR
        put(note(root,Q*0.9,.55),t0); put(note(root-12,Q*0.9,.35),t0)
        for m in chord: put(note(m-12,Q*0.5,.18),t0+Q)
        # melody: eighths with some sixteenths, stepwise within the chord scale
        scale=SC[c]; pos=0.0; idx=min(range(len(scale)),key=lambda i:abs(scale[i]-prev))
        shape=rng.choice([[.5,.5,.5,.5],[.5,.25,.25,.5,.5],[1,.5,.5],[.5,.5,1]])
        for d in shape:
            idx=max(0,min(len(scale)-1,idx+rng.choice([-1,1,1,-2,2,0] if s!='B' else [-1,1,1,1,-2])))
            m=scale[idx]; prev=m
            put(note(m,d*Q*0.85,.42),t0+pos*Q); pos+=d
        bar+=1
# coda
t0=bar*BAR
for k,m in enumerate([69,72,76,81]): put(note(m,0.3,.45),t0+k*0.14)
put(note(45,2.5,.6),t0+0.56); put(note(57,2.5,.5),t0+0.56); put(note(64,2.5,.3),t0+0.56); put(note(69,2.5,.35),t0+0.56)
# tiny room: two soft echoes
out=out+0.18*np.roll(out,int(.043*SR))+0.1*np.roll(out,int(.087*SR))
out/=np.abs(out).max()/0.85
fade=int(3*SR); out[-fade:]*=np.linspace(1,0,fade)
pcm=(out*32767).astype('<i2').tobytes()
open('galop.raw','wb').write(pcm)
print('encode: ffmpeg -f s16le -ar 32000 -ac 1 -i galop.raw -t 116 -af afade=t=out:st=112:d=4 -c:a libvorbis -q:a 2 assets/demo/silent-cinema-galop.ogg')
