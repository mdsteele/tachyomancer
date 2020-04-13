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

use super::super::super::button::{HoverPulse, TextButton};
use super::super::super::paragraph::{Paragraph, StreamingParagraph};
use crate::mancer::gui::{Event, Keycode, Resources, Sound, Ui};
use crate::mancer::save::Prefs;
use crate::mancer::state::{Cutscene, Portrait};
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, Rect};
use tachy::save::{Puzzle, PuzzleKind};

//===========================================================================//

const BUBBLE_FONT_SIZE: f32 = 20.0;
const BUBBLE_INNER_MARGIN: i32 = 12;
const BUBBLE_LINE_HEIGHT: f32 = 22.0;

const CHOICE_BUTTON_SPACING: i32 = 2;

const CUTSCENE_BUTTON_HEIGHT: i32 = 50;

const PORTRAIT_HEIGHT: i32 = 85;
const PORTRAIT_WIDTH: i32 = 68;

const PUZZLE_BUTTON_HEIGHT: i32 = 50;
const PUZZLE_BUTTON_SPACING: i32 = 3;

//===========================================================================//

pub enum BubbleAction {
    GoToPuzzle(Puzzle),
    MakeChoice(String, String),
    ParagraphFinished,
    PlayCutscene(Cutscene),
}

pub enum ReachedAction {
    PlayCutscene(Cutscene),
    UnlockPuzzles(Vec<Puzzle>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BubbleKind {
    Choice,
    Cutscene,
    Puzzle,
    Speech,
}

//===========================================================================//

pub trait BubbleView {
    fn kind(&self) -> BubbleKind;

    fn rect(&self) -> Rect<i32>;

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

    fn on_first_reached(&self) -> Option<ReachedAction> {
        None
    }

    fn skip_paragraph(&mut self, _ui: &mut Ui) {}

    fn should_pause_afterwards(&self) -> bool {
        false
    }

    fn is_finished(&self) -> bool;
}

//===========================================================================//

pub struct CutsceneBubbleView {
    button: TextButton<Cutscene>,
}

impl CutsceneBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        cutscene: Cutscene,
    ) -> Box<dyn BubbleView> {
        let rect = Rect::new(0, top, width, CUTSCENE_BUTTON_HEIGHT);
        Box::new(CutsceneBubbleView {
            button: TextButton::new(rect, "Replay cutscene", cutscene),
        })
    }
}

impl BubbleView for CutsceneBubbleView {
    fn kind(&self) -> BubbleKind {
        BubbleKind::Cutscene
    }

    fn rect(&self) -> Rect<i32> {
        self.button.rect()
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        self.button.draw(resources, matrix, true);
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        self.button.on_event(event, ui, true).map(BubbleAction::PlayCutscene)
    }

    fn on_first_reached(&self) -> Option<ReachedAction> {
        Some(ReachedAction::PlayCutscene(*self.button.value()))
    }

    fn is_finished(&self) -> bool {
        true
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
        let height = (PUZZLE_BUTTON_HEIGHT + PUZZLE_BUTTON_SPACING)
            * num_puzzles
            - PUZZLE_BUTTON_SPACING;
        Box::new(PuzzleBubbleView {
            rect: Rect::new(0, top, width, height),
            buttons: puzzles
                .into_iter()
                .enumerate()
                .map(|(index, puzzle)| {
                    let y = top
                        + (PUZZLE_BUTTON_HEIGHT + PUZZLE_BUTTON_SPACING)
                            * (index as i32);
                    let label = if puzzle.kind() == PuzzleKind::Sandbox {
                        format!("Go to \"{}\" sandbox", puzzle.title())
                    } else {
                        format!("Go to task \"{}\"", puzzle.title())
                    };
                    TextButton::new(
                        Rect::new(0, y, width, PUZZLE_BUTTON_HEIGHT),
                        &label,
                        puzzle,
                    )
                })
                .collect(),
        })
    }
}

impl BubbleView for PuzzleBubbleView {
    fn kind(&self) -> BubbleKind {
        BubbleKind::Puzzle
    }

    fn rect(&self) -> Rect<i32> {
        self.rect
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

    fn on_first_reached(&self) -> Option<ReachedAction> {
        Some(ReachedAction::UnlockPuzzles(
            self.buttons.iter().map(|button| *button.value()).collect(),
        ))
    }

    fn is_finished(&self) -> bool {
        true
    }
}

//===========================================================================//

pub struct SpeechBubbleView {
    rect: Rect<i32>,
    portrait: Option<Portrait>,
    paragraph: StreamingParagraph,
    paragraph_left_top: (f32, f32),
    sent_finished: bool,
    pause_after: bool,
}

impl SpeechBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        portrait: Option<Portrait>,
        prefs: &Prefs,
        format: &str,
        pause_after: bool,
    ) -> Box<dyn BubbleView> {
        let (height, paragraph, paragraph_left_top) = if portrait.is_some() {
            let wrap_width = width - PORTRAIT_WIDTH - 3 * BUBBLE_INNER_MARGIN;
            let paragraph = Paragraph::compile(
                BUBBLE_FONT_SIZE,
                BUBBLE_LINE_HEIGHT,
                wrap_width as f32,
                prefs,
                format,
            );
            let height = (PORTRAIT_HEIGHT + 2 * BUBBLE_INNER_MARGIN).max(
                2 * BUBBLE_INNER_MARGIN + (paragraph.height().ceil() as i32),
            );
            let paragraph_left = PORTRAIT_WIDTH + 2 * BUBBLE_INNER_MARGIN;
            let paragraph_top = top + BUBBLE_INNER_MARGIN;
            (height, paragraph, (paragraph_left as f32, paragraph_top as f32))
        } else {
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
            (height, paragraph, (paragraph_left, paragraph_top))
        };
        Box::new(SpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            portrait,
            paragraph: StreamingParagraph::new(paragraph),
            paragraph_left_top,
            sent_finished: false,
            pause_after,
        })
    }

    fn action(&mut self) -> Option<BubbleAction> {
        if !self.sent_finished && self.paragraph.is_done() {
            self.sent_finished = true;
            Some(BubbleAction::ParagraphFinished)
        } else {
            None
        }
    }
}

impl BubbleView for SpeechBubbleView {
    fn kind(&self) -> BubbleKind {
        BubbleKind::Speech
    }

    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if let Some(portrait) = self.portrait {
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
                portrait as u32,
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
        } else {
            // Draw bubble:
            resources.shaders().ui().draw_bubble(
                matrix,
                &self.rect.as_f32(),
                &Color4::ORANGE2,
                &Color4::ORANGE1,
                &Color4::ORANGE0_TRANSLUCENT,
            );
        }
        self.paragraph.draw(resources, matrix, self.paragraph_left_top);
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        match event {
            Event::ClockTick(tick) => {
                self.paragraph.on_clock_tick(tick, ui);
                return self.action();
            }
            Event::MouseDown(mouse) if mouse.left => {
                if self.rect.contains_point(mouse.pt) {
                    self.paragraph.skip_to_end(ui);
                    return self.action();
                }
            }
            Event::KeyDown(key) if key.code == Keycode::Return => {
                self.paragraph.skip_to_end(ui);
                return self.action();
            }
            Event::Unfocus => {
                self.paragraph.skip_to_end(ui);
            }
            _ => {}
        }
        return None;
    }

    fn skip_paragraph(&mut self, ui: &mut Ui) {
        self.paragraph.skip_to_end(ui);
        self.sent_finished = true;
    }

    fn should_pause_afterwards(&self) -> bool {
        self.pause_after
    }

    fn is_finished(&self) -> bool {
        self.sent_finished
    }
}

//===========================================================================//

pub struct YouChoiceBubbleView {
    rect: Rect<i32>,
    key: String,
    choices: Vec<ChoiceButton>,
}

impl YouChoiceBubbleView {
    pub fn new(
        width: i32,
        top: i32,
        prefs: &Prefs,
        key: String,
        choices: Vec<(String, String)>,
    ) -> Box<dyn BubbleView> {
        debug_assert!(!choices.is_empty());
        let mut bottom = top - CHOICE_BUTTON_SPACING;
        let num_choices = choices.len();
        let choices = choices
            .into_iter()
            .enumerate()
            .map(|(index, (value, format))| {
                let choice = ChoiceButton::new(
                    width,
                    bottom + CHOICE_BUTTON_SPACING,
                    ChoicePosition::from_index_and_count(index, num_choices),
                    prefs,
                    value,
                    &format,
                );
                bottom = choice.rect.bottom();
                choice
            })
            .collect();
        Box::new(YouChoiceBubbleView {
            rect: Rect::new(0, top, width, bottom - top),
            key,
            choices,
        })
    }
}

impl BubbleView for YouChoiceBubbleView {
    fn kind(&self) -> BubbleKind {
        BubbleKind::Choice
    }

    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for choice in self.choices.iter() {
            choice.draw(resources, matrix);
        }
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        for choice in self.choices.iter_mut() {
            if let Some(value) = choice.on_event(event, ui) {
                return Some(BubbleAction::MakeChoice(
                    self.key.clone(),
                    value,
                ));
            }
        }
        return None;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

//===========================================================================//

#[derive(Clone, Copy)]
enum ChoicePosition {
    First,
    Middle,
    Last,
}

impl ChoicePosition {
    fn from_index_and_count(index: usize, count: usize) -> ChoicePosition {
        if index == 0 {
            ChoicePosition::First
        } else if index + 1 == count {
            ChoicePosition::Last
        } else {
            ChoicePosition::Middle
        }
    }

    fn bubble_kind(self) -> u32 {
        match self {
            ChoicePosition::First => 1,
            ChoicePosition::Middle => 2,
            ChoicePosition::Last => 3,
        }
    }
}

//===========================================================================//

struct ChoiceButton {
    rect: Rect<i32>,
    position: ChoicePosition,
    paragraph: Paragraph,
    paragraph_left_top: (f32, f32),
    value: String,
    hover_pulse: HoverPulse,
}

impl ChoiceButton {
    fn new(
        width: i32,
        top: i32,
        position: ChoicePosition,
        prefs: &Prefs,
        value: String,
        format: &str,
    ) -> ChoiceButton {
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
        ChoiceButton {
            rect: Rect::new(0, top, width, height),
            position,
            paragraph,
            paragraph_left_top: (paragraph_left, paragraph_top),
            value,
            hover_pulse: HoverPulse::new(),
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let bg_color = Color4::CYAN0_TRANSLUCENT
            .mix(Color4::CYAN3_TRANSLUCENT, self.hover_pulse.brightness());
        resources.shaders().ui().draw_bubble_kind(
            matrix,
            &self.rect.as_f32(),
            self.position.bubble_kind(),
            &Color4::CYAN3,
            &Color4::CYAN2,
            &bg_color,
        );
        self.paragraph.draw(resources, matrix, self.paragraph_left_top);
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) -> Option<String> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick, ui);
            }
            Event::MouseDown(mouse) => {
                if mouse.left && self.rect.contains_point(mouse.pt) {
                    self.hover_pulse.on_click(ui);
                    ui.audio().play_sound(Sound::ButtonClick);
                    return Some(self.value.clone());
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if self.hover_pulse.set_hovering(hovering, ui) {
                    ui.audio().play_sound(Sound::ButtonHover);
                }
            }
            Event::Unfocus => self.hover_pulse.unfocus(),
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
