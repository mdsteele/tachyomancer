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

use std::collections::VecDeque;
use std::mem;
use tachy::geom::Color4;
use tachy::gui::{Sound, Ui};

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

fn intro_cutscene() -> CutsceneScript {
    CutsceneScript::new(sn::seq(vec![
        sn::background(0.5, 0.0, 0.0),
        sn::wait(1.0),
        sn::background(0.0, 0.5, 0.0),
        sn::par(vec![
            sn::seq(vec![
                sn::wait(0.5),
                sn::sound(Sound::Beep),
                sn::background(0.0, 0.5, 0.5),
            ]),
            sn::seq(vec![
                sn::pause(),
                sn::background(0.0, 0.0, 0.5),
                sn::wait(0.5),
                sn::sound(Sound::Beep),
            ]),
        ]),
    ]))
}

//===========================================================================//

pub struct CutsceneScript {
    node: SceneNode,
}

impl CutsceneScript {
    fn new(node: SceneNode) -> CutsceneScript { CutsceneScript { node } }

    pub fn is_paused(&self) -> bool { self.node.is_paused() }

    pub fn unpause(&mut self) { self.node.unpause() }

    pub fn skip(&mut self, theater: &mut Theater) { self.node.skip(theater); }

    pub fn tick(&mut self, elapsed: f64, ui: &mut Ui, theater: &mut Theater)
                -> bool {
        self.node.tick(elapsed, ui, theater).is_some()
    }
}

//===========================================================================//

pub struct Theater {
    bg_color: Color4,
}

impl Theater {
    pub fn new() -> Theater {
        Theater { bg_color: Color4::new(0.0, 0.0, 0.0, 1.0) }
    }

    pub fn background_color(&self) -> Color4 { self.bg_color }
}

//===========================================================================//

mod sn {
    use super::SceneNode;
    use tachy::geom::Color4;
    use tachy::gui::Sound;

    pub(super) fn seq(nodes: Vec<SceneNode>) -> SceneNode {
        SceneNode::Seq(nodes.into())
    }

    pub(super) fn par(nodes: Vec<SceneNode>) -> SceneNode {
        SceneNode::Par(nodes)
    }

    pub(super) fn background(r: f32, g: f32, b: f32) -> SceneNode {
        SceneNode::Background(Color4::new(r, g, b, 1.0), false)
    }

    pub(super) fn pause() -> SceneNode { SceneNode::Pause(false) }

    pub(super) fn sound(sound: Sound) -> SceneNode {
        SceneNode::Sound(sound, false)
    }

    pub(super) fn wait(seconds: f64) -> SceneNode { SceneNode::Wait(seconds) }
}

//===========================================================================//

pub(self) enum SceneNode {
    Seq(VecDeque<SceneNode>),
    Par(Vec<SceneNode>),
    Background(Color4, bool),
    Pause(bool),
    Sound(Sound, bool),
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
            &SceneNode::Pause(done) => !done,
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
            &mut SceneNode::Pause(ref mut done) => {
                *done = true;
            }
            _ => {}
        }
    }

    fn skip(&mut self, theater: &mut Theater) {
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
                theater.bg_color = color;
                *done = true;
            }
            &mut SceneNode::Pause(ref mut done) => {
                *done = true;
            }
            &mut SceneNode::Sound(_, ref mut done) => {
                *done = true;
            }
            &mut SceneNode::Wait(ref mut duration) => {
                *duration = 0.0;
            }
        }
    }

    fn tick(&mut self, elapsed: f64, ui: &mut Ui, theater: &mut Theater)
            -> Option<f64> {
        match self {
            &mut SceneNode::Seq(ref mut nodes) => {
                let mut remaining = elapsed;
                while !nodes.is_empty() {
                    if let Some(remain) = nodes
                        .front_mut()
                        .unwrap()
                        .tick(remaining, ui, theater)
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
                    if let Some(remain) = node.tick(elapsed, ui, theater) {
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
                    theater.bg_color = color;
                    *done = true;
                }
                Some(elapsed)
            }
            &mut SceneNode::Pause(done) => {
                if done { Some(elapsed) } else { None }
            }
            &mut SceneNode::Sound(sound, ref mut done) => {
                if !*done {
                    ui.audio().play_sound(sound);
                    *done = true;
                }
                Some(elapsed)
            }
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
