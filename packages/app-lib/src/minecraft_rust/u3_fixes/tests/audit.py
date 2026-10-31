#!/usr/bin/env python3
"""CPU regressions for production TNT velocity response and instrument codec.
Not an in-game, renderer, sound-device, network or platform smoke test.
"""
from pathlib import Path
import subprocess,tempfile,sys
p=Path(__file__).resolve().parents[1];engine=Path(sys.argv[1])
def fn(text,name):
 a=text.index('fn '+name+'(');b=text.index('{',a);level=1;i=b+1
 while level:
  level+=(text[i]=='{')-(text[i]=='}');i+=1
 return text[a:i]
tnt=(engine/'crates/horizon-client/src/game/tnt.rs').read_text();a=tnt.index('        self.on_ground = wanted.y');b=tnt.index('        self.fuse -= 1;',a)
baseline='fn finish_motion(wanted:Vec3,moved:Vec3,velocity:Vec3)->(Vec3,bool){ let mut tnt=Tnt{velocity,on_ground:false};'+tnt[a:b].replace('self.','tnt.')+'(tnt.velocity,tnt.on_ground)}'
fixed=fn((p/'tnt_runtime.rs').read_text(),'finish_motion')
prelude='''#![allow(dead_code)]
#[derive(Clone,Copy,Debug)]struct Vec3{x:f64,y:f64,z:f64}
impl Vec3{const ZERO:Self=Self{x:0.,y:0.,z:0.};fn new(x:f64,y:f64,z:f64)->Self{Self{x,y,z}}}
struct Tnt{velocity:Vec3,on_ground:bool}
'''
tests='''
#[test]fn falling_tnt_bounces_instead_of_zeroing_impact_velocity(){let v=Vec3::new(0.2,-0.5,0.1);let(r,g)=finish_motion(v,Vec3::new(0.2,-0.1,0.1),v);assert!(g);assert!((r.y-0.245).abs()<1e-12);assert!((r.x-0.1372).abs()<1e-12);}
#[test]fn unimpeded_air_motion_keeps_drag(){let v=Vec3::new(0.1,-0.2,0.3);let(r,g)=finish_motion(v,v,v);assert!(!g);assert!((r.y+0.196).abs()<1e-12);}
#[test]fn ceiling_cancels_upward_motion(){let v=Vec3::new(0.0,0.2,0.0);let(r,g)=finish_motion(v,Vec3::ZERO,v);assert!(!g);assert_eq!(r.y,0.0);}
#[test]fn wall_cancels_only_its_horizontal_axis(){let v=Vec3::new(0.2,-0.1,0.3);let(r,g)=finish_motion(v,Vec3::new(0.0,-0.1,0.3),v);assert!(!g);assert_eq!(r.x,0.0);assert!((r.z-0.294).abs()<1e-12);}
'''
h=(p/'horn.rs').read_text();codec=h[h.index('#[derive'):h.index('fn resolve(')]
nbt='''#![allow(dead_code)]
use std::collections::BTreeMap;
#[derive(Clone)]struct Text(String);impl Text{fn to_string_lossy(&self)->String{self.0.clone()}}
#[derive(Clone)]enum Tag{Int(i32),Float(u32),Double(u64),String(Text),Compound(BTreeMap<String,Tag>)}
impl Tag{fn get(&self,k:&str)->Option<&Tag>{if let Self::Compound(m)=self{m.get(k)}else{None}}}
fn text(s:&str)->Tag{Tag::String(Text(s.into()))}
fn map(v:Vec<(&str,Tag)>)->Tag{Tag::Compound(v.into_iter().map(|(k,v)|(k.into(),v)).collect())}
fn instrument(duration:Tag,range:Tag,sound:Tag)->Tag{map(vec![("use_duration",duration),("range",range),("sound_event",sound)])}
fn normal()->Tag{instrument(Tag::Double(7f64.to_bits()),Tag::Int(256),text("minecraft:item.goat_horn.sound.0"))}
'''
h_tests='''
#[test]fn default_range_and_seven_second_use(){let i=decode(&normal()).unwrap();assert_eq!(i.ticks,140);assert_eq!(i.volume,16.0);assert_eq!(i.sound,"minecraft:item.goat_horn.sound.0");}
#[test]fn inline_sound_and_float_duration(){let t=instrument(Tag::Float(3f32.to_bits()),Tag::Float(64f32.to_bits()),map(vec![("sound_id",text("example:horn"))]));let i=decode(&t).unwrap();assert_eq!(i.ticks,60);assert_eq!(i.volume,4.0);assert_eq!(i.sound,"example:horn");}
#[test]fn malformed_duration_cannot_overflow(){for n in [f64::NAN,f64::INFINITY,-1.0,0.0,1e30]{let t=instrument(Tag::Double(n.to_bits()),Tag::Int(256),text("test:horn"));assert!(decode(&t).is_none());}}
#[test]fn invalid_gain_is_rejected(){for n in [f64::NAN,f64::INFINITY,-1.0,1e30]{let t=instrument(Tag::Int(7),Tag::Double(n.to_bits()),text("test:horn"));assert!(decode(&t).is_none());}}
#[test]fn missing_fields_do_not_invent_a_sound(){assert!(decode(&map(vec![])).is_none());}
#[test]fn unknown_sound_schema_is_rejected(){assert!(decode(&instrument(Tag::Int(7),Tag::Int(256),Tag::Int(0))).is_none());}
#[test]fn zero_range_is_a_valid_silent_instrument(){let i=decode(&instrument(Tag::Int(7),Tag::Int(0),text("test:horn"))).unwrap();assert_eq!(i.volume,0.0);}
#[test]fn sub_tick_duration_is_rejected(){assert!(decode(&instrument(Tag::Double(0.001f64.to_bits()),Tag::Int(16),text("test:horn"))).is_none());}
'''
with tempfile.TemporaryDirectory(prefix='u3-audit-') as tmp:
 for name,code in [('tnt-before',prelude+baseline+tests),('tnt-after',prelude+fixed+tests),('horn-codec',nbt+codec.replace('pub(super) ','')+h_tests)]:
  path=Path(tmp)/(name+'.rs');path.write_text(code);exe=path.with_suffix('')
  subprocess.run(['rustc','--edition=2024','--test',str(path),'-o',str(exe)],check=True)
  r=subprocess.run([str(exe)],capture_output=True,text=True)
  if name=='tnt-before':
   assert r.returncode and '1 failed' in r.stdout and '3 passed' in r.stdout,r.stdout+r.stderr
   print('TNT baseline: missing ground bounce reproduced.')
  else:
   print(name+':\n'+r.stdout);assert r.returncode==0,r.stdout+r.stderr
