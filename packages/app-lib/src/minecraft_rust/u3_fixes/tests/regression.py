#!/usr/bin/env python3
"""CPU regression harness: real pinned ThrownTrident impl + U3 transition code.
Geometry, item payload and projectile world are small test doubles. This is NOT
an in-game or GPU test. cargo check --tests additionally checks the real types.
"""
import pathlib, subprocess, tempfile, sys
engine = pathlib.Path(sys.argv[1])
patch = pathlib.Path(__file__).resolve().parents[1]
source = (engine / 'crates/horizon-client/src/game/trident.rs').read_text()
trident = source[source.index('pub(super) struct ThrownTrident'):source.index('const HALF:')]
arrow = (engine / 'crates/horizon-client/src/arrows.rs').read_text()
back_off = arrow[arrow.index('pub fn back_off('):arrow.index('fn bounds(', arrow.index('pub fn back_off('))]
runtime = (patch / 'runtime.rs').read_text()
motion = runtime[runtime.index('pub(super) fn authoritative_motion'):runtime.index('// AbstractArrow.shouldFall')]
support = runtime[runtime.index('fn support_result'):runtime.index('#[cfg(test)]')]
tests = runtime[runtime.index('#[cfg(test)]'):]
prelude = r'''
#![allow(dead_code, private_interfaces)]
#[derive(Clone, Copy)] struct Vec3 { x:f64, y:f64, z:f64 }
impl Vec3 {
 const ZERO:Self=Self{x:0.,y:0.,z:0.};
 fn new(x:f64,y:f64,z:f64)->Self{Self{x,y,z}}
 fn add_vector(self,b:Self)->Self{Self::new(self.x+b.x,self.y+b.y,self.z+b.z)}
 fn scale(self,k:f64)->Self{Self::new(self.x*k,self.y*k,self.z*k)}
}
struct Aabb;
struct ItemStack { item:Box<str> }
impl ItemStack { fn new(id:&str,_count:u32)->Self{Self{item:id.into()}} }
mod horizon_client {
 pub mod mobs { pub struct Who; }
 pub mod projectiles {
  use crate::{Vec3,Aabb,horizon_client::mobs::Who};
  pub(crate) trait ProjectileWorld {
   fn clip(&self,from:Vec3,to:Vec3)->Option<(Vec3,[i32;3],[i32;3])>;
   fn in_water(&self,bounds:Aabb)->bool;
   fn targets(&self)->&[(Who,Aabb)];
  }
 }
}
use horizon_client::projectiles::ProjectileWorld;
'''
with tempfile.TemporaryDirectory(prefix='u3-trident-') as tmp:
    d=pathlib.Path(tmp)
    prefix = prelude + back_off + '\nmod subject { use super::*;\n' + trident
    fixed = prefix + motion + support + tests + '\n}'
    baseline = (prefix + 'fn authoritative_motion(_: &mut ThrownTrident, _: [f64;3]) {}\n'
                + 'fn support_result(_: &mut ThrownTrident, _: Option<bool>) {}\n' + tests + '\n}')
    for name, text in [('baseline',baseline),('fixed',fixed)]:
        src=d/(name+'.rs'); src.write_text(text)
        exe=d/name
        subprocess.run(['rustc','--edition=2024','--test',str(src),'-o',str(exe)],check=True)
        r=subprocess.run([str(exe),'--nocapture'],capture_output=True,text=True)
        if name=='baseline':
            assert r.returncode != 0, 'Baseline unexpectedly passes; regression was not reproduced'
            assert '2 failed' in r.stdout and '5 passed' in r.stdout, r.stdout+r.stderr
            print('BASELINE: two expected failures reproduced (server motion, removed support).')
        else:
            print(r.stdout)
            assert r.returncode == 0, r.stdout+r.stderr
