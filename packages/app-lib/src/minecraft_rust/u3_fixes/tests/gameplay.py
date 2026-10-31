#!/usr/bin/env python3
"""CPU regressions using pinned game method bodies and production patch functions.
World, protocol transport and menus are small doubles, not an in-game/GPU test.
The CI mixin transform + cargo check checks actual Horizon types separately.
"""
import pathlib, subprocess, tempfile, sys
engine = pathlib.Path(sys.argv[1])/'crates/horizon-client/src'
patch = pathlib.Path(__file__).resolve().parents[1]
def function(text, name):
    start=text.index('fn '+name+'(')
    begin=text.index('{',start)
    depth=1; end=begin+1
    while depth:
        depth += (text[end]=='{')-(text[end]=='}'); end+=1
    return text[start:end]
game=(engine/'game.rs').read_text()
mount=(engine/'game/mount_menu.rs').read_text()
original='\n'.join([function((engine/'game/remote.rs').read_text(),'remote_handle_input'),function(mount,'open_ridden_mount_menu'),function(mount,'riding_mount_with_menu'),function(game,'tick_sleep')])
fixed=original.replace('fn remote_handle_input(&mut self) {','fn remote_handle_input(&mut self) { patch::stop_menu_use(self);').replace('fn open_ridden_mount_menu(&mut self) -> bool {','fn open_ridden_mount_menu(&mut self) -> bool { if patch::open_ridden_chest(self) { return true; }').replace('fn riding_mount_with_menu(&self) -> bool {','fn riding_mount_with_menu(&self) -> bool { if patch::has_ridden_chest(self) { return true; }')
fixed=fixed.replace(function(game,'tick_sleep'),'fn tick_sleep(&mut self) { patch::tick_sleep(self); }')
prelude=r'''
#![allow(dead_code,unused_imports,private_interfaces)]
use std::collections::{HashMap,HashSet};
use std::marker::PhantomData;
#[derive(Clone,Copy,PartialEq,Eq,Debug,Hash)] enum KeyCode{ShiftLeft,KeyF,KeyQ,KeyE}
#[derive(Clone,Copy,PartialEq,Eq,Debug)] enum Screen{Playing,Inventory,Creative,Chat}
#[derive(Clone,Copy,Default)] struct Vec3{x:f64,y:f64,z:f64}
impl Vec3 { const ZERO:Self=Self{x:0.,y:0.,z:0.}; fn new(x:f64,y:f64,z:f64)->Self{Self{x,y,z}} }
mod horizon_core {pub mod geometry{pub(crate) use crate::Vec3;}}
#[derive(Clone)] struct Stack{item:String}
#[derive(Clone,Debug,PartialEq,Eq)] enum BlockEntity{Chest(Vec<Option<u32>>)}
mod horizon_client{pub mod containers{pub(crate) use crate::BlockEntity;}}
mod horizon_protocol{pub mod packets{pub mod play{
#[derive(Debug)]pub struct PlayerAction{pub action:i32,pub pos:[i32;3],pub direction:i32,pub sequence:i32}
#[derive(Debug)]pub struct PlayerCommand{pub id:i32,pub action:i32,pub data:i32}
#[derive(Debug)]pub struct SetCarriedItem{pub slot:i16}
#[derive(Debug)]pub enum Serverbound{PlayerAction(PlayerAction),PlayerCommand(PlayerCommand),SetCarriedItem(SetCarriedItem)}
}}}
use horizon_protocol::packets::play::{self,Serverbound as S};
#[derive(Default)] struct Input;
impl Input{fn tick(&mut self,_:&mut RemoteInputWorld<'_,'_>,_:bool,_:bool){}}
#[derive(Default)] struct Remote{input:Input,stop_mining:bool,loaded_sent:bool,carried:Option<usize>,pending_keys:Vec<KeyCode>,entity_id:i32}
struct RemoteInputWorld<'a,'b>(&'a mut Game<'b>);
struct Boat{id:u32,chest:bool,items:Vec<Option<u32>>}
struct Mob{extra:Extra}
struct Extra{tame:bool}
impl Mob{fn species(&self)->u8{0} fn alive(&self)->bool{true}}
fn has_mount_menu(_:u8)->bool{true}
mod minecarts{pub const CART_KEY_Y:i32=-2147483648;}
struct Health{dead:bool}
impl Health{fn dead_or_dying(&self)->bool{self.dead}}
struct Player{position:Vec3,velocity:Vec3,health:Health}
#[derive(Default)] struct Weather{raining:bool,thundering:bool,rain_time:i32,thunder_time:i32}
impl Weather{fn levels(&self,_:f32)->(f32,f32){(0.,0.)}}
#[derive(Default)] struct BlockEntities(HashMap<(i32,i32,i32),BlockEntity>);
impl std::ops::Deref for BlockEntities{type Target=HashMap<(i32,i32,i32),BlockEntity>;fn deref(&self)->&Self::Target{&self.0}}
impl BlockEntities{fn insert(&mut self,k:(i32,i32,i32),v:BlockEntity){self.0.insert(k,v);}}
#[derive(Default)] struct World{block_entities:BlockEntities,day_time:i64,weather:Weather,removed:bool}
impl World{fn block(&self,_:i32,_:i32,_:i32)->usize{if self.removed{1}else{0}}}
struct Facts{name:String}
struct Res;
impl Res{fn facts(&self)->&Self{self} fn get(&self,id:usize)->Facts{Facts{name:if id==0{"minecraft:straw_bed"}else{"minecraft:air"}.into()}}}
struct Rules;
impl Rules{fn bool(&self,_:&str)->bool{true}}
struct Ui{rules:Rules}
#[derive(Default)] struct Clicks{miss_time:u32}
struct Tutorial;
impl Tutorial{fn open_inventory(&self)->bool{false}}
struct Game<'a>{remote:Option<Remote>,screen:Screen,using:Option<(usize,u32)>,riding:Option<u32>,boats:Vec<Boat>,mobs:Vec<Mob>,world:World,station:Option<()>,craft_size:u32,container:Option<(i32,i32,i32)>,right:bool,left:bool,selected:usize,keys:HashSet<KeyCode>,sleeping:Option<[i32;3]>,sleep_timer:u32,player:Option<Player>,res:Res,ui:Ui,sleep_seen:(u32,u32),clicks:Clicks,sent:std::cell::RefCell<Vec<S>>,stack:Option<Stack>,riptide:u32,spec:bool,creative:bool,tutorial:Tutorial,lifetime:PhantomData<&'a ()>}
impl Default for Game<'_>{fn default()->Self{Self{remote:None,screen:Screen::Playing,using:None,riding:None,boats:vec![],mobs:vec![],world:World::default(),station:None,craft_size:2,container:None,right:false,left:false,selected:0,keys:HashSet::new(),sleeping:None,sleep_timer:0,player:Some(Player{position:Vec3::ZERO,velocity:Vec3::ZERO,health:Health{dead:false}}),res:Res,ui:Ui{rules:Rules},sleep_seen:(0,0),clicks:Clicks::default(),sent:std::cell::RefCell::new(vec![]),stack:None,riptide:0,spec:false,creative:false,tutorial:Tutorial,lifetime:PhantomData}}}
impl Game<'_>{
fn send(&self,s:S){self.sent.borrow_mut().push(s)}
fn used_hand(&self)->Option<usize>{self.using.map(|u|u.0)}
fn stack_in(&self,_:usize)->Option<&Stack>{self.stack.as_ref()}
fn with_hand(&mut self,_:usize,f:impl FnOnce(&mut Self)){f(self)}
fn release_trident(&mut self,_:u32){self.riptide+=1}
fn spectator(&self)->bool{self.spec}
fn capture(&mut self,_:bool){}
fn mount_mob_index(&self,_:u32)->Option<usize>{None}
fn open_mount_menu(&mut self,_:usize){}
fn stop_remote_mining(&mut self){}
fn remote_swap_hands(&mut self){}
fn has_control_down(&self)->bool{false}
fn remote_drop(&mut self,_:bool){}
fn enough_slept(&self)->bool{self.sleeping.is_some()&&self.sleep_timer>=100}
fn wake_up(&mut self){self.sleeping=None;self.sleep_timer=100;}
fn wake_guests(&mut self){}
}
'''
tests=r'''
#[cfg(test)] mod tests{use super::*;
fn boat()->Game<'static>{let mut g=Game::default();g.riding=Some(42);g.boats.push(Boat{id:42,chest:true,items:vec![Some(7),None]});g}
fn eating()->Game<'static>{let mut g=Game::default();g.remote=Some(Remote{loaded_sent:true,..Default::default()});g.screen=Screen::Inventory;g.using=Some((0,20));g.stack=Some(Stack{item:"minecraft:apple".into()});g}
#[test]fn menu_releases_food_exactly_once(){let mut g=eating();g.remote_handle_input();g.remote_handle_input();assert!(g.using.is_none());let s=g.sent.borrow();assert_eq!(s.len(),1);assert!(matches!(s[0],S::PlayerAction(play::PlayerAction{action:6,..})));}
#[test]fn gameplay_does_not_cancel_held_food(){let mut g=eating();g.screen=Screen::Playing;g.remote_handle_input();assert!(g.using.is_some());}
#[test]fn remote_menu_preserves_riptide_release(){let mut g=eating();g.stack.as_mut().unwrap().item="minecraft:trident".into();g.remote_handle_input();assert_eq!(g.riptide,1);}
#[test]fn ridden_chest_is_detected(){assert!(boat().riding_mount_with_menu());}
#[test]fn boat_menu_uses_vehicle_container(){let mut g=boat();assert!(g.open_ridden_mount_menu());assert_eq!(g.screen,Screen::Inventory);let key=g.container.unwrap();assert_eq!(key.2,42);assert_eq!(g.world.block_entities[&key],BlockEntity::Chest(vec![Some(7),None]));}
#[test]fn reopening_keeps_newest_slots(){let mut g=boat();let key=(i32::MAX-15,minecarts::CART_KEY_Y,42);g.world.block_entities.insert(key,BlockEntity::Chest(vec![Some(99)]));assert!(g.open_ridden_mount_menu());assert_eq!(g.world.block_entities[&key],BlockEntity::Chest(vec![Some(99)]));}
#[test]fn ordinary_boat_is_not_a_chest(){let mut g=boat();g.boats[0].chest=false;assert!(!g.riding_mount_with_menu());assert!(!g.open_ridden_mount_menu());}
#[test]fn remote_never_fabricates_inventory(){let mut g=boat();g.remote=Some(Remote::default());assert!(!patch::open_ridden_chest(&mut g));assert!(g.container.is_none());}
#[test]fn sneaking_into_straw_bed_stays_until_morning(){let mut g=Game::default();g.sleeping=Some([0,64,0]);g.world.day_time=14000;g.keys.insert(KeyCode::ShiftLeft);for _ in 0..99{g.tick_sleep();assert!(g.sleeping.is_some());}g.tick_sleep();assert!(g.sleeping.is_none());assert_eq!(g.world.day_time,24000);}
#[test]fn broken_bed_still_wakes_player(){let mut g=Game::default();g.sleeping=Some([0,64,0]);g.world.removed=true;g.tick_sleep();assert!(g.sleeping.is_none());assert_eq!(g.world.day_time,0);}
}
'''
payload=(patch/'gameplay.rs').read_text()
# Test the exact production code; doubles stand in for renderer/world/protocol only.
module='mod patch{use crate::{horizon_client,horizon_core,horizon_protocol};'+payload+'}\n'
timer=function(game,'tick_sleep_timer')
with tempfile.TemporaryDirectory(prefix='u3-gameplay-') as tmp:
    for name,body in [('baseline',original),('fixed',fixed)]:
        text=prelude+timer+module+"impl Game<'_>{"+body+'}\n'+tests
        src=pathlib.Path(tmp)/(name+'.rs');src.write_text(text);exe=src.with_suffix('')
        subprocess.run(['rustc','--edition=2024','--test',str(src),'-o',str(exe)],check=True)
        out=subprocess.run([str(exe)],capture_output=True,text=True)
        if name=='baseline':
            assert out.returncode and '6 failed' in out.stdout and '4 passed' in out.stdout,out.stdout+out.stderr
            print('BASELINE: six failures reproduced: menu release, Riptide, boat detection/open/reopen, Shift sleep.')
        else:
            print(out.stdout);assert out.returncode==0,out.stdout+out.stderr
