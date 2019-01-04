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

use super::list::ListView;
use cgmath::{Matrix4, Point2, vec2, vec3};
use num_integer::{div_floor, div_mod_floor};
use std::borrow::Cow;
use tachy::font::Align;
use tachy::geom::Rect;
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources};
use tachy::save::{Conversation, Puzzle};
use tachy::state::{ConversationBubble, ConversationPortrait, GameState};

//===========================================================================//

const CONV_LIST_WIDTH: i32 = 280;
const ELEMENT_SPACING: i32 = 22;

const BUBBLE_FONT_SIZE: f32 = 20.0;
const BUBBLE_INNER_MARGIN: i32 = 12;
const BUBBLE_LINE_HEIGHT: f32 = 22.0;
const BUBBLE_SPACING: i32 = 16;

const CHOICE_HEIGHT: i32 = 30;
const CHOICE_SPACING: i32 = 2;

const PORTRAIT_HEIGHT: i32 = 75;
const PORTRAIT_WIDTH: i32 = 60;

const PUZZLE_BUBBLE_HEIGHT: i32 = 50;

const SCROLLBAR_WIDTH: i32 = 15;
const SCROLLBAR_MARGIN: i32 = 5;

//===========================================================================//

#[derive(Clone)]
pub enum ConverseAction {
    GoToPuzzle(Puzzle),
    MakeChoice(String, String),
}

//===========================================================================//

pub struct ConverseView {
    conv_list: ListView<Conversation>,
    bubbles_list: BubblesListView,
}

impl ConverseView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> ConverseView {
        ConverseView {
            conv_list: ListView::new(Rect::new(rect.x,
                                               rect.y,
                                               CONV_LIST_WIDTH,
                                               rect.height),
                                     &state.current_conversation(),
                                     conv_list_items(state)),
            bubbles_list: BubblesListView::new(Rect::new(rect.x +
                                                             CONV_LIST_WIDTH +
                                                             ELEMENT_SPACING,
                                                         rect.y,
                                                         rect.width -
                                                             CONV_LIST_WIDTH -
                                                             ELEMENT_SPACING,
                                                         rect.height),
                                               state),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        let conv = state.current_conversation();
        self.conv_list.draw(resources, matrix, &conv);
        self.bubbles_list.draw(resources, matrix);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<ConverseAction> {
        if let Some(conv) =
            self.conv_list.handle_event(event, &state.current_conversation())
        {
            state.set_current_conversation(conv);
        }
        self.bubbles_list.handle_event(event)
    }

    pub fn update_conversation_bubbles(&mut self, state: &GameState) {
        self.bubbles_list.update_conversation(state);
    }

    pub fn update_conversation_list(&mut self, state: &GameState) {
        self.conv_list
            .set_items(&state.current_conversation(), conv_list_items(state));
    }

    pub fn unfocus(&mut self) {
        self.conv_list.unfocus();
        self.bubbles_list.unfocus();
    }
}

fn conv_list_items(state: &GameState) -> Vec<(Conversation, String)> {
    Conversation::all()
        .filter(|&conv| state.is_conversation_unlocked(conv))
        .map(|conv| {
                 let mut label = conv.title().to_string();
                 if !state.is_conversation_complete(conv) {
                     label = format!("* {}", label);
                 }
                 (conv, label)
             })
        .collect()
}

//===========================================================================//

struct BubblesListView {
    rect: Rect<i32>,
    bubbles: Vec<Box<BubbleView>>,
    scroll_top: i32,
    scroll_max: i32,
    drag: Option<i32>,
}

impl BubblesListView {
    fn new(rect: Rect<i32>, state: &GameState) -> BubblesListView {
        let mut view = BubblesListView {
            rect,
            bubbles: Vec::new(),
            scroll_top: 0,
            scroll_max: 0,
            drag: None,
        };
        view.update_conversation(state);
        view.scroll_top = view.scroll_max;
        view
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw background and define clipping area:
        let stencil = Stencil::new();
        {
            let color = (0.1, 0.1, 0.1);
            resources
                .shaders()
                .solid()
                .fill_rect(&matrix, color, self.rect.as_f32());
        }
        stencil.enable_clipping();

        // Draw conversation bubbles:
        let bubble_offset = vec3(self.rect.x as f32,
                                 (self.rect.y - self.scroll_top) as f32,
                                 0.0);
        let bubble_matrix = matrix * Matrix4::from_translation(bubble_offset);
        for bubble in self.bubbles.iter() {
            let rect = bubble.rect();
            if rect.bottom() > self.scroll_top &&
                rect.y < self.scroll_top + self.rect.height
            {
                bubble.draw(resources, &bubble_matrix);
            }
        }

        // Draw scrollbar:
        if let Some(handle_rect) = self.scroll_handle_rect() {
            let color = (0.3, 0.1, 0.3);
            let rect = Rect::new((self.rect.right() - SCROLLBAR_WIDTH) as f32,
                                 self.rect.y as f32,
                                 SCROLLBAR_WIDTH as f32,
                                 self.rect.height as f32);
            resources.shaders().solid().fill_rect(&matrix, color, rect);
            let color = if self.drag.is_some() {
                (0.9, 0.6, 0.9)
            } else {
                (0.9, 0.1, 0.9)
            };
            resources
                .shaders()
                .solid()
                .fill_rect(&matrix, color, handle_rect.as_f32());
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Option<ConverseAction> {
        // Handle scrollbar events:
        match event {
            Event::MouseDown(mouse)
                if mouse.left && self.rect.contains_point(mouse.pt) => {
                if let Some(handle_rect) = self.scroll_handle_rect() {
                    if handle_rect.contains_point(mouse.pt) {
                        self.drag = Some(mouse.pt.y - handle_rect.y);
                    }
                    // TODO: support jumping up/down page
                }
            }
            Event::MouseMove(mouse) => {
                if let Some(drag_offset) = self.drag {
                    let new_handle_y = mouse.pt.y - drag_offset - self.rect.y;
                    let total_height = self.scroll_max + self.rect.height;
                    let new_scroll_top = div_round(total_height *
                                                       new_handle_y,
                                                   self.rect.height);
                    self.scroll_top =
                        new_scroll_top.max(0).min(self.scroll_max);
                }
            }
            Event::MouseUp(mouse) if mouse.left => {
                self.drag = None;
            }
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                let new_scroll_top = self.scroll_top - scroll.delta.y;
                self.scroll_top = new_scroll_top.max(0).min(self.scroll_max);
            }
            _ => {}
        }

        // Handle conversation bubble events:
        let bubble_event =
            event.relative_to(self.rect.top_left() - vec2(0, self.scroll_top));
        for bubble in self.bubbles.iter_mut() {
            if let Some(action) = bubble.handle_event(&bubble_event) {
                return Some(action);
            }
        }
        return None;
    }

    pub fn update_conversation(&mut self, state: &GameState) {
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        let conv = profile.current_conversation();
        let bubble_width = self.rect.width -
            (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
        let mut bubble_top: i32 = 0;
        let bubble_seq = ConversationBubble::sequence(conv, profile);
        let mut bubble_views = Vec::with_capacity(bubble_seq.len());
        for bubble in bubble_seq {
            if bubble_top > 0 {
                bubble_top += BUBBLE_SPACING;
            }
            let bubble_view = match bubble {
                ConversationBubble::NpcSpeech(portrait, text) => {
                    NpcSpeechBubbleView::new(bubble_width,
                                             bubble_top,
                                             portrait,
                                             &text)
                }
                ConversationBubble::Puzzle(puzzle) => {
                    PuzzleBubbleView::new(bubble_width, bubble_top, puzzle)
                }
                ConversationBubble::YouChoice(key, choices) => {
                    YouChoiceBubbleView::new(bubble_width,
                                             bubble_top,
                                             key,
                                             choices)
                }
                ConversationBubble::YouSpeech(text) => {
                    YouSpeechBubbleView::new(bubble_width, bubble_top, &text)
                }
            };
            bubble_top += bubble_view.rect().height;
            bubble_views.push(bubble_view);
        }
        self.bubbles = bubble_views;
        self.scroll_max = (bubble_top - self.rect.height).max(0);
    }

    fn unfocus(&mut self) {
        self.drag = None;
        for bubble in self.bubbles.iter_mut() {
            bubble.unfocus();
        }
    }

    fn scroll_handle_rect(&self) -> Option<Rect<i32>> {
        if self.scroll_max != 0 {
            let total_height = self.scroll_max + self.rect.height;
            Some(Rect::new(self.rect.right() - SCROLLBAR_WIDTH,
                           self.rect.y +
                               div_round(self.rect.height * self.scroll_top,
                                         total_height),
                           SCROLLBAR_WIDTH,
                           div_round(self.rect.height * self.rect.height,
                                     total_height)))
        } else {
            None
        }
    }
}

//===========================================================================//

trait BubbleView {
    fn rect(&self) -> Rect<i32>;

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>);

    fn handle_event(&mut self, _event: &Event) -> Option<ConverseAction> {
        None
    }

    fn unfocus(&mut self) {}
}

//===========================================================================//

struct NpcSpeechBubbleView {
    rect: Rect<i32>,
    portrait: ConversationPortrait,
    lines: Vec<String>,
}

impl NpcSpeechBubbleView {
    fn new(width: i32, top: i32, portrait: ConversationPortrait, text: &str)
           -> Box<BubbleView> {
        let num_cols = div_floor(width - PORTRAIT_WIDTH -
                                     3 * BUBBLE_INNER_MARGIN,
                                 (0.5 * BUBBLE_FONT_SIZE) as i32)
            .max(1) as usize;
        let lines = textwrap::wrap_iter(text, num_cols)
            .map(Cow::into_owned)
            .collect::<Vec<String>>();
        let height = (PORTRAIT_HEIGHT + 2 * BUBBLE_INNER_MARGIN)
            .max(2 * BUBBLE_INNER_MARGIN +
                     ((lines.len() as f32) * BUBBLE_LINE_HEIGHT -
                          (BUBBLE_LINE_HEIGHT - BUBBLE_FONT_SIZE))
                         .ceil() as i32);
        let view = NpcSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            portrait,
            lines,
        };
        Box::new(view)
    }
}

impl BubbleView for NpcSpeechBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw bubble:
        let rect = self.rect.as_f32();
        let color = (0.1, 0.5, 0.1);
        resources.shaders().solid().fill_rect(&matrix, color, rect);

        // Draw portrait:
        let portrait_rect = Rect::new(self.rect.x + BUBBLE_INNER_MARGIN,
                                      self.rect.y + BUBBLE_INNER_MARGIN,
                                      PORTRAIT_WIDTH,
                                      PORTRAIT_HEIGHT);
        let portrait_rect = portrait_rect.as_f32();
        let color = (0.3, 0.5, 0.3);
        resources.shaders().solid().fill_rect(&matrix, color, portrait_rect);
        resources.fonts().roman().draw(matrix,
                                       BUBBLE_FONT_SIZE,
                                       Align::MidCenter,
                                       (portrait_rect.x +
                                            0.5 * portrait_rect.width,
                                        portrait_rect.y +
                                            0.5 * portrait_rect.height),
                                       &format!("{:?}", self.portrait));

        // Draw text:
        let left = (self.rect.x + PORTRAIT_WIDTH +
            2 * BUBBLE_INNER_MARGIN) as f32;
        let mut top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        for string in self.lines.iter() {
            resources.fonts().roman().draw(matrix,
                                           BUBBLE_FONT_SIZE,
                                           Align::TopLeft,
                                           (left, top),
                                           string);
            top += BUBBLE_LINE_HEIGHT;
        }
    }
}

//===========================================================================//

struct PuzzleBubbleView {
    rect: Rect<i32>,
    puzzle: Puzzle,
    hovering: bool,
}

impl PuzzleBubbleView {
    fn new(width: i32, top: i32, puzzle: Puzzle) -> Box<BubbleView> {
        let view = PuzzleBubbleView {
            rect: Rect::new(0, top, width, PUZZLE_BUBBLE_HEIGHT),
            puzzle,
            hovering: false,
        };
        Box::new(view)
    }
}

impl BubbleView for PuzzleBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = if self.hovering {
            (0.1, 1.0, 1.0)
        } else {
            (0.1, 0.5, 0.5)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        let label = format!("Go to task \"{}\"", self.puzzle.title());
        resources.fonts().roman().draw(&matrix,
                                       BUBBLE_FONT_SIZE,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       &label);
    }

    fn handle_event(&mut self, event: &Event) -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return Some(ConverseAction::GoToPuzzle(self.puzzle));
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            _ => {}
        }
        return None;
    }

    fn unfocus(&mut self) { self.hovering = false; }
}

//===========================================================================//

struct YouChoiceBubbleView {
    rect: Rect<i32>,
    key: String,
    choices: Vec<(String, String)>,
    hovering: Option<usize>,
}

impl YouChoiceBubbleView {
    fn new(width: i32, top: i32, key: String, choices: Vec<(String, String)>)
           -> Box<BubbleView> {
        debug_assert!(!choices.is_empty());
        let height = (choices.len() as i32) *
            (CHOICE_HEIGHT + CHOICE_SPACING) -
            CHOICE_SPACING;
        let view = YouChoiceBubbleView {
            rect: Rect::new(0, top, width, height),
            key,
            choices,
            hovering: None,
        };
        Box::new(view)
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
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for (index, &(_, ref label)) in self.choices.iter().enumerate() {
            let color = if self.hovering == Some(index) {
                (1.0, 1.0, 0.1)
            } else {
                (0.5, 0.5, 0.1)
            };
            let rect = Rect::new(self.rect.x,
                                 self.rect.y +
                                     (index as i32) *
                                         (CHOICE_HEIGHT + CHOICE_SPACING),
                                 self.rect.width,
                                 CHOICE_HEIGHT)
                .as_f32();
            resources.shaders().solid().fill_rect(&matrix, color, rect);
            resources.fonts().roman().draw(&matrix,
                                           BUBBLE_FONT_SIZE,
                                           Align::MidRight,
                                           (rect.x + rect.width -
                                                BUBBLE_INNER_MARGIN as f32,
                                            rect.y + 0.5 * rect.height),
                                           &label);
        }
    }

    fn handle_event(&mut self, event: &Event) -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if let Some(index) = self.choice_for_pt(mouse.pt) {
                    let key = self.key.clone();
                    let value = self.choices[index].0.clone();
                    return Some(ConverseAction::MakeChoice(key, value));
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.choice_for_pt(mouse.pt);
            }
            _ => {}
        }
        return None;
    }

    fn unfocus(&mut self) { self.hovering = None; }
}

//===========================================================================//

struct YouSpeechBubbleView {
    rect: Rect<i32>,
    lines: Vec<String>,
}

impl YouSpeechBubbleView {
    fn new(width: i32, top: i32, text: &str) -> Box<BubbleView> {
        let num_cols = div_floor(width - 2 * BUBBLE_INNER_MARGIN,
                                 (0.5 * BUBBLE_FONT_SIZE) as i32)
            .max(1) as usize;
        let lines = textwrap::wrap_iter(text, num_cols)
            .map(Cow::into_owned)
            .collect::<Vec<String>>();
        let height = 2 * BUBBLE_INNER_MARGIN +
            ((lines.len() as f32) * BUBBLE_LINE_HEIGHT -
                 (BUBBLE_LINE_HEIGHT - BUBBLE_FONT_SIZE))
                .ceil() as i32;
        let view = YouSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            lines,
        };
        Box::new(view)
    }
}

impl BubbleView for YouSpeechBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = (0.5, 0.1, 0.1);
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, color, self.rect.as_f32());
        let right = (self.rect.right() - BUBBLE_INNER_MARGIN) as f32;
        let mut top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        for string in self.lines.iter() {
            resources.fonts().roman().draw(matrix,
                                           BUBBLE_FONT_SIZE,
                                           Align::TopRight,
                                           (right, top),
                                           string);
            top += BUBBLE_LINE_HEIGHT;
        }
    }
}

//===========================================================================//

fn div_round(a: i32, b: i32) -> i32 {
    ((a as f64) / (b as f64)).round() as i32
}

//===========================================================================//
