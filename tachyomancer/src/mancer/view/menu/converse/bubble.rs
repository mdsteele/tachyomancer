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

use super::super::super::button::TextButton;
use super::super::super::paragraph::{Paragraph, StreamingParagraph};
use crate::mancer::font::Align;
use crate::mancer::gui::{Event, Keycode, Resources, Ui};
use crate::mancer::save::Prefs;
use crate::mancer::state::{Cutscene, Portrait};
use cgmath::{Matrix4, Point2};
use num_integer::div_mod_floor;
use tachy::geom::{AsFloat, Color3, Color4, Rect};
use tachy::save::Puzzle;

//===========================================================================//

const BUBBLE_FONT_SIZE: f32 = 20.0;
const BUBBLE_INNER_MARGIN: i32 = 12;
const BUBBLE_LINE_HEIGHT: f32 = 22.0;

const CHOICE_HEIGHT: i32 = 30;
const CHOICE_SPACING: i32 = 2;

const CUTSCENE_BUBBLE_HEIGHT: i32 = 50;

const PORTRAIT_HEIGHT: i32 = 85;
const PORTRAIT_WIDTH: i32 = 68;

const PUZZLE_HEIGHT: i32 = 50;
const PUZZLE_SPACING: i32 = 3;

//===========================================================================//

#[derive(Clone)]
pub enum BubbleAction {
    Complete,
    GoToPuzzle(Puzzle),
    Increment,
    MakeChoice(String, String),
    PlayCutscene(Cutscene),
}

//===========================================================================//

pub trait BubbleView {
    fn rect(&self) -> Rect<i32>;

    fn is_choice_or_puzzle(&self) -> bool {
        false
    }

    fn has_puzzle(&self, _puzzle: Puzzle) -> bool {
        false
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>);

    fn on_event(
        &mut self,
        _event: &Event,
        _ui: &mut Ui,
    ) -> Option<BubbleAction> {
        None
    }
}

//===========================================================================//

pub struct CutsceneBubbleView {
    rect: Rect<i32>,
    button: TextButton<BubbleAction>,
}

impl CutsceneBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        cutscene: Cutscene,
    ) -> Box<dyn BubbleView> {
        let rect = Rect::new(0, top, width, CUTSCENE_BUBBLE_HEIGHT);
        Box::new(CutsceneBubbleView {
            rect,
            button: TextButton::new(
                rect,
                "Replay cutscene",
                BubbleAction::PlayCutscene(cutscene),
            ),
        })
    }
}

impl BubbleView for CutsceneBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        self.button.draw(resources, matrix, true);
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        self.button.on_event(event, ui, true)
    }
}

//===========================================================================//

pub struct NpcSpeechBubbleView {
    rect: Rect<i32>,
    portrait: Portrait,
    paragraph: StreamingParagraph,
}

impl NpcSpeechBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        portrait: Portrait,
        prefs: &Prefs,
        format: &str,
    ) -> Box<dyn BubbleView> {
        let wrap_width = width - PORTRAIT_WIDTH - 3 * BUBBLE_INNER_MARGIN;
        let paragraph = Paragraph::compile(
            BUBBLE_FONT_SIZE,
            BUBBLE_LINE_HEIGHT,
            wrap_width as f32,
            prefs,
            format,
        );
        let height = (PORTRAIT_HEIGHT + 2 * BUBBLE_INNER_MARGIN)
            .max(2 * BUBBLE_INNER_MARGIN + (paragraph.height().ceil() as i32));
        Box::new(NpcSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            portrait,
            paragraph: StreamingParagraph::new(paragraph),
        })
    }
}

impl BubbleView for NpcSpeechBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw bubble:
        resources.shaders().ui().draw_bubble(
            matrix,
            &self.rect.as_f32(),
            &Color4::PURPLE2,
            &Color4::ORANGE1,
            &Color4::PURPLE0_TRANSLUCENT,
        );

        // Draw portrait:
        let portrait_left_top = Point2::new(
            self.rect.x + BUBBLE_INNER_MARGIN,
            self.rect.y + BUBBLE_INNER_MARGIN,
        );
        resources.shaders().portrait().draw(
            matrix,
            self.portrait as u32,
            portrait_left_top.as_f32(),
            resources.textures().portraits(),
        );

        // Draw portrait frame:
        let frame_rect = Rect::new(
            self.rect.x + BUBBLE_INNER_MARGIN - 2,
            self.rect.y + BUBBLE_INNER_MARGIN - 2,
            72,
            89,
        );
        resources.shaders().ui().draw_list_frame(
            matrix,
            &frame_rect.as_f32(),
            &Color4::PURPLE2,
            &Color4::PURPLE1,
            &Color4::PURPLE0,
        );

        // Draw paragraph:
        let left =
            (self.rect.x + PORTRAIT_WIDTH + 2 * BUBBLE_INNER_MARGIN) as f32;
        let top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        match event {
            Event::ClockTick(tick) => {
                self.paragraph.tick(tick.elapsed, ui);
            }
            Event::MouseDown(mouse) if mouse.left => {
                if self.rect.contains_point(mouse.pt) {
                    self.paragraph.skip_to_end(ui);
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Return => {
                self.paragraph.skip_to_end(ui);
            }
            Event::Unfocus => {
                self.paragraph.skip_to_end(ui);
            }
            _ => {}
        }
        None
    }
}

//===========================================================================//

pub struct PuzzleBubbleView {
    rect: Rect<i32>,
    buttons: Vec<TextButton<Puzzle>>,
}

impl PuzzleBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        puzzles: Vec<Puzzle>,
    ) -> Box<dyn BubbleView> {
        let num_puzzles = puzzles.len() as i32;
        debug_assert!(num_puzzles > 0);
        let height =
            (PUZZLE_HEIGHT + PUZZLE_SPACING) * num_puzzles - PUZZLE_SPACING;
        Box::new(PuzzleBubbleView {
            rect: Rect::new(0, top, width, height),
            buttons: puzzles
                .into_iter()
                .enumerate()
                .map(|(index, puzzle)| {
                    let y = top
                        + (PUZZLE_HEIGHT + PUZZLE_SPACING) * (index as i32);
                    TextButton::new(
                        Rect::new(0, y, width, PUZZLE_HEIGHT),
                        &format!("Go to task \"{}\"", puzzle.title()),
                        puzzle,
                    )
                })
                .collect(),
        })
    }
}

impl BubbleView for PuzzleBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn is_choice_or_puzzle(&self) -> bool {
        true
    }

    fn has_puzzle(&self, puzzle: Puzzle) -> bool {
        self.buttons.iter().any(|button| *button.value() == puzzle)
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for button in self.buttons.iter() {
            button.draw(resources, matrix, true);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        for button in self.buttons.iter_mut() {
            if let Some(puzzle) = button.on_event(event, ui, true) {
                return Some(BubbleAction::GoToPuzzle(puzzle));
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct YouChoiceBubbleView {
    rect: Rect<i32>,
    key: String,
    choices: Vec<(String, String)>,
    hovering: Option<usize>,
}

impl YouChoiceBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        key: String,
        choices: Vec<(String, String)>,
    ) -> Box<dyn BubbleView> {
        debug_assert!(!choices.is_empty());
        let height = (choices.len() as i32) * (CHOICE_HEIGHT + CHOICE_SPACING)
            - CHOICE_SPACING;
        Box::new(YouChoiceBubbleView {
            rect: Rect::new(0, top, width, height),
            key,
            choices,
            hovering: None,
        })
    }

    fn choice_for_pt(&self, pt: Point2<i32>) -> Option<usize> {
        if self.rect.contains_point(pt) {
            let rel_y = pt.y - self.rect.y;
            let (index, offset) =
                div_mod_floor(rel_y, CHOICE_HEIGHT + CHOICE_SPACING);
            if offset < CHOICE_HEIGHT {
                debug_assert!(index >= 0);
                let index = index as usize;
                debug_assert!(index < self.choices.len());
                return Some(index);
            }
        }
        return None;
    }
}

impl BubbleView for YouChoiceBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn is_choice_or_puzzle(&self) -> bool {
        true
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for (index, &(_, ref label)) in self.choices.iter().enumerate() {
            let color = if self.hovering == Some(index) {
                Color3::new(1.0, 1.0, 0.1)
            } else {
                Color3::new(0.5, 0.5, 0.1)
            };
            let rect = Rect::new(
                self.rect.x,
                self.rect.y
                    + (index as i32) * (CHOICE_HEIGHT + CHOICE_SPACING),
                self.rect.width,
                CHOICE_HEIGHT,
            )
            .as_f32();
            resources.shaders().solid().fill_rect(&matrix, color, rect);
            resources.fonts().roman().draw(
                &matrix,
                BUBBLE_FONT_SIZE,
                Align::MidRight,
                (
                    rect.x + rect.width - BUBBLE_INNER_MARGIN as f32,
                    rect.y + 0.5 * rect.height,
                ),
                &label,
            );
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        match event {
            Event::MouseDown(mouse) => {
                if let Some(index) = self.choice_for_pt(mouse.pt) {
                    let key = self.key.clone();
                    let value = self.choices[index].0.clone();
                    return Some(BubbleAction::MakeChoice(key, value));
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.choice_for_pt(mouse.pt);
                if self.hovering != hovering {
                    self.hovering = hovering;
                    ui.request_redraw();
                }
            }
            Event::Unfocus => {
                if self.hovering.is_some() {
                    self.hovering = None;
                    ui.request_redraw();
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//

pub struct YouSpeechBubbleView {
    rect: Rect<i32>,
    paragraph: StreamingParagraph,
    paragraph_left_top: (f32, f32),
}

impl YouSpeechBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        prefs: &Prefs,
        format: &str,
    ) -> Box<dyn BubbleView> {
        let wrap_width = width - 2 * BUBBLE_INNER_MARGIN;
        let paragraph = Paragraph::compile(
            BUBBLE_FONT_SIZE,
            BUBBLE_LINE_HEIGHT,
            wrap_width as f32,
            prefs,
            format,
        );
        let height =
            2 * BUBBLE_INNER_MARGIN + (paragraph.height().ceil() as i32);
        let paragraph_right = (width - BUBBLE_INNER_MARGIN) as f32;
        let paragraph_left = (paragraph_right - paragraph.width()).floor();
        let paragraph_top = (top + BUBBLE_INNER_MARGIN) as f32;
        Box::new(YouSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            paragraph: StreamingParagraph::new(paragraph),
            paragraph_left_top: (paragraph_left, paragraph_top),
        })
    }
}

impl BubbleView for YouSpeechBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        resources.shaders().ui().draw_bubble(
            matrix,
            &self.rect.as_f32(),
            &Color4::ORANGE2,
            &Color4::ORANGE1,
            &Color4::ORANGE0_TRANSLUCENT,
        );
        self.paragraph.draw(resources, matrix, self.paragraph_left_top);
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        match event {
            Event::ClockTick(tick) => {
                self.paragraph.tick(tick.elapsed, ui);
            }
            Event::MouseDown(mouse) if mouse.left => {
                if self.rect.contains_point(mouse.pt) {
                    self.paragraph.skip_to_end(ui);
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Return => {
                self.paragraph.skip_to_end(ui);
            }
            Event::Unfocus => {
                self.paragraph.skip_to_end(ui);
            }
            _ => {}
        }
        None
    }
}

//===========================================================================//
