// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

use super::converse::Portrait;
use crate::tachy::geom::Color3;
use crate::tachy::gui::Sound;
use std::collections::VecDeque;
use std::mem;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum Cutscene {
    Intro,
}

impl Cutscene {
    pub fn script(&self) -> CutsceneScript {
        match *self {
            Cutscene::Intro => intro_cutscene(),
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn intro_cutscene() -> CutsceneScript {
    CutsceneScript::new(sn::seq(vec![
        sn::background(0.1, 0.1, 0.1),
        sn::wait(1.0),
        sn::par(vec![
            sn::talk(Portrait::Esra, (-40, -50), "Wake up..."),
            sn::sound(Sound::ButtonHover), // TODO: heartbeat
        ]),
        sn::wait(0.5),
        sn::talk(Portrait::Lisa, (0, 0), "\
            Commander $'YOURNAME', is it?  I'm Captain Lisa Jackson.  It's \
            good to have you on as my first officer on this mission."),
        sn::wait(1.0),
        sn::par(vec![
            sn::talk(Portrait::Esra, (40, 50), "Wake up, $'YOURNAME'..."),
            sn::seq(vec![
                sn::sound(Sound::ButtonHover), // TODO: heartbeat
                sn::wait(1.0),
                sn::sound(Sound::ButtonHover), // TODO: heartbeat
            ]),
        ]),
        sn::wait(0.5),
        sn::talk(Portrait::Lisa, (0, 0), "\
            I have a feeling that your skills will come in handy."),
        sn::wait(1.0),
    ]))
}

//===========================================================================//

pub trait Theater {
    fn add_talk(
        &mut self,
        portrait: Portrait,
        pos: (i32, i32),
        format: &str,
    ) -> i32;

    fn talk_is_done(&self, tag: i32) -> bool;

    fn remove_talk(&mut self, tag: i32);

    fn play_sound(&mut self, sound: Sound);

    fn set_background_color(&mut self, color: Color3);
}

//===========================================================================//

pub struct CutsceneScript {
    node: SceneNode,
}

impl CutsceneScript {
    fn new(node: SceneNode) -> CutsceneScript {
        CutsceneScript { node }
    }

    pub fn is_paused(&self) -> bool {
        self.node.is_paused()
    }

    pub fn unpause(&mut self) {
        self.node.unpause()
    }

    pub fn skip<T: Theater>(&mut self, theater: &mut T) {
        self.node.skip(theater);
    }

    pub fn tick<T: Theater>(&mut self, elapsed: f64, theater: &mut T) -> bool {
        self.node.tick(elapsed, theater).is_some()
    }
}

//===========================================================================//

mod sn {
    use super::super::converse::Portrait;
    use super::{SceneNode, TalkPhase};
    use crate::tachy::geom::Color3;
    use crate::tachy::gui::Sound;

    pub(super) fn seq(nodes: Vec<SceneNode>) -> SceneNode {
        SceneNode::Seq(nodes.into())
    }

    pub(super) fn par(nodes: Vec<SceneNode>) -> SceneNode {
        SceneNode::Par(nodes)
    }

    pub(super) fn background(r: f32, g: f32, b: f32) -> SceneNode {
        SceneNode::Background(Color3::new(r, g, b), false)
    }

    pub(super) fn sound(sound: Sound) -> SceneNode {
        SceneNode::Sound(sound, false)
    }

    pub(super) fn talk(
        portrait: Portrait,
        pos: (i32, i32),
        format: &'static str,
    ) -> SceneNode {
        SceneNode::Talk(TalkPhase::Unstarted(portrait, pos, format))
    }

    pub(super) fn wait(seconds: f64) -> SceneNode {
        SceneNode::Wait(seconds)
    }
}

//===========================================================================//

pub(self) enum TalkPhase {
    Unstarted(Portrait, (i32, i32), &'static str),
    Active(i32),
    Paused(i32),
    Cleanup(i32),
    Finished,
}

//===========================================================================//

pub(self) enum SceneNode {
    Seq(VecDeque<SceneNode>),
    Par(Vec<SceneNode>),
    Background(Color3, bool),
    Sound(Sound, bool),
    Talk(TalkPhase),
    Wait(f64),
}

impl SceneNode {
    fn is_paused(&self) -> bool {
        match self {
            &SceneNode::Seq(ref nodes) => {
                nodes.front().map(|node| node.is_paused()).unwrap_or(false)
            }
            &SceneNode::Par(ref nodes) => {
                nodes.iter().any(|node| node.is_paused())
            }
            &SceneNode::Talk(TalkPhase::Paused(_)) => true,
            _ => false,
        }
    }

    fn unpause(&mut self) {
        match self {
            &mut SceneNode::Seq(ref mut nodes) => {
                if let Some(node) = nodes.front_mut() {
                    node.unpause();
                }
            }
            &mut SceneNode::Par(ref mut nodes) => {
                for node in nodes.iter_mut() {
                    node.unpause();
                }
            }
            &mut SceneNode::Talk(ref mut phase) => {
                if let TalkPhase::Paused(tag) = *phase {
                    *phase = TalkPhase::Cleanup(tag);
                }
            }
            _ => {}
        }
    }

    fn skip<T: Theater>(&mut self, theater: &mut T) {
        match self {
            &mut SceneNode::Seq(ref mut nodes) => {
                for node in nodes.iter_mut() {
                    node.skip(theater);
                }
                nodes.clear();
            }
            &mut SceneNode::Par(ref mut nodes) => {
                for node in nodes.iter_mut() {
                    node.skip(theater);
                }
                nodes.clear();
            }
            &mut SceneNode::Background(color, ref mut done) => {
                theater.set_background_color(color);
                *done = true;
            }
            &mut SceneNode::Sound(_, ref mut done) => {
                *done = true;
            }
            &mut SceneNode::Talk(ref mut phase) => {
                match *phase {
                    TalkPhase::Active(tag)
                    | TalkPhase::Paused(tag)
                    | TalkPhase::Cleanup(tag) => theater.remove_talk(tag),
                    _ => {}
                }
                *phase = TalkPhase::Finished;
            }
            &mut SceneNode::Wait(ref mut duration) => {
                *duration = 0.0;
            }
        }
    }

    fn tick<T: Theater>(
        &mut self,
        elapsed: f64,
        theater: &mut T,
    ) -> Option<f64> {
        match self {
            &mut SceneNode::Seq(ref mut nodes) => {
                let mut remaining = elapsed;
                while !nodes.is_empty() {
                    if let Some(remain) =
                        nodes.front_mut().unwrap().tick(remaining, theater)
                    {
                        remaining = remain;
                        nodes.pop_front();
                    } else {
                        return None;
                    }
                }
                Some(remaining)
            }
            &mut SceneNode::Par(ref mut nodes) => {
                let mut min_remaining = Some(elapsed);
                for mut node in mem::replace(nodes, Vec::new()) {
                    if let Some(remain) = node.tick(elapsed, theater) {
                        min_remaining = min_remaining.map(|r| r.min(remain));
                    } else {
                        min_remaining = None;
                        nodes.push(node);
                    }
                }
                min_remaining
            }
            &mut SceneNode::Background(color, ref mut done) => {
                if !*done {
                    theater.set_background_color(color);
                    *done = true;
                }
                Some(elapsed)
            }
            &mut SceneNode::Sound(sound, ref mut done) => {
                if !*done {
                    theater.play_sound(sound);
                    *done = true;
                }
                Some(elapsed)
            }
            &mut SceneNode::Talk(ref mut phase) => match *phase {
                TalkPhase::Unstarted(portrait, pos, format) => {
                    let tag = theater.add_talk(portrait, pos, format);
                    *phase = TalkPhase::Active(tag);
                    None
                }
                TalkPhase::Active(tag) => {
                    if theater.talk_is_done(tag) {
                        *phase = TalkPhase::Paused(tag);
                    }
                    None
                }
                TalkPhase::Paused(_) => None,
                TalkPhase::Cleanup(tag) => {
                    theater.remove_talk(tag);
                    *phase = TalkPhase::Finished;
                    Some(elapsed)
                }
                TalkPhase::Finished => Some(elapsed),
            },
            &mut SceneNode::Wait(ref mut duration) => {
                if *duration > elapsed {
                    *duration -= elapsed;
                    None
                } else {
                    let remaining = elapsed - *duration;
                    *duration = 0.0;
                    Some(remaining)
                }
            }
        }
    }
}

//===========================================================================//
