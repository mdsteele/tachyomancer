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

use super::button::Scrollbar;
use super::list::ListView;
use cgmath::{Matrix4, Point2, vec2};
use num_integer::{div_floor, div_mod_floor};
use std::borrow::Cow;
use tachy::font::Align;
use tachy::geom::{AsFloat, MatrixExt, Rect};
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources};
use tachy::save::{Conversation, Profile, Puzzle};
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

const MORE_BUTTON_HEIGHT: i32 = 30;

const PORTRAIT_HEIGHT: i32 = 75;
const PORTRAIT_WIDTH: i32 = 60;

const PUZZLE_BUBBLE_HEIGHT: i32 = 50;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 5;

//===========================================================================//

#[derive(Clone)]
pub enum ConverseAction {
    Complete,
    GoToPuzzle(Puzzle),
    Increment,
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

    pub fn on_event(&mut self, event: &Event, state: &mut GameState)
                    -> Option<ConverseAction> {
        if let Some(conv) =
            self.conv_list.on_event(event, &state.current_conversation())
        {
            state.set_current_conversation(conv);
            self.update_conversation_bubbles(state);
            None
        } else {
            self.bubbles_list.on_event(event)
        }
    }

    pub fn update_conversation_bubbles(&mut self, state: &GameState) {
        self.bubbles_list.update_conversation(state);
    }

    pub fn update_conversation_list(&mut self, state: &GameState) {
        self.conv_list
            .set_items(&state.current_conversation(), conv_list_items(state));
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
    conv: Conversation,
    bubbles: Vec<Box<BubbleView>>,
    num_bubbles_shown: usize,
    more_button: Option<MoreButton>,
    scrollbar: Scrollbar,
}

impl BubblesListView {
    fn new(rect: Rect<i32>, state: &GameState) -> BubblesListView {
        let scrollbar_rect = Rect::new(rect.right() - SCROLLBAR_WIDTH,
                                       rect.y,
                                       SCROLLBAR_WIDTH,
                                       rect.height);
        let mut view = BubblesListView {
            rect,
            conv: Conversation::first(),
            bubbles: Vec::new(),
            num_bubbles_shown: 0,
            more_button: None,
            scrollbar: Scrollbar::new(scrollbar_rect),
        };
        view.update_conversation(state);
        view
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
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
        let scroll_top = self.scrollbar.scroll_top();
        let bubble_matrix = matrix *
            Matrix4::trans2(self.rect.x as f32,
                            (self.rect.y - scroll_top) as f32);
        for bubble in self.bubbles.iter().take(self.num_bubbles_shown) {
            let rect = bubble.rect();
            if rect.bottom() > scroll_top &&
                rect.y < scroll_top + self.rect.height
            {
                bubble.draw(resources, &bubble_matrix);
            }
        }
        if let Some(ref button) = self.more_button {
            let rect = button.rect;
            if rect.bottom() > scroll_top &&
                rect.y < scroll_top + self.rect.height
            {
                button.draw(resources, &bubble_matrix);
            }
        }

        // Draw scrollbar:
        self.scrollbar.draw(resources, matrix);
    }

    fn on_event(&mut self, event: &Event) -> Option<ConverseAction> {
        // Handle scrollbar events:
        self.scrollbar.on_event(event);
        match event {
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y);
            }
            _ => {}
        }

        // Handle conversation bubble events:
        let bubble_event =
            event.relative_to(self.rect.top_left() -
                                  vec2(0, self.scrollbar.scroll_top()));
        for bubble in self.bubbles.iter_mut().take(self.num_bubbles_shown) {
            if let Some(action) = bubble.on_event(&bubble_event) {
                return Some(action);
            }
        }
        if let Some(ref mut button) = self.more_button {
            if button.on_event(&bubble_event) {
                if self.num_bubbles_shown + 1 < self.bubbles.len() {
                    return Some(ConverseAction::Increment);
                }
                if let Some(ref bubble) = self.bubbles.last() {
                    if bubble.is_choice_or_puzzle() {
                        return Some(ConverseAction::Increment);
                    }
                }
                return Some(ConverseAction::Complete);
            }
        }
        return None;
    }

    fn update_conversation(&mut self, state: &GameState) {
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        let conv = profile.current_conversation();
        let num_bubbles_shown =
            profile.conversation_progress(conv).saturating_add(1);
        if conv != self.conv || num_bubbles_shown > self.bubbles.len() {
            self.rebuild_bubbles(profile, conv);
        }
        self.num_bubbles_shown = num_bubbles_shown.min(self.bubbles.len());

        self.more_button = if self.num_bubbles_shown < self.bubbles.len() {
            let width = self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
            let top = if self.num_bubbles_shown == 0 {
                0
            } else {
                self.bubbles[self.num_bubbles_shown - 1].rect().bottom() +
                    BUBBLE_SPACING
            };
            let more_button = MoreButton::new(width, top);
            Some(more_button)
        } else {
            None
        };

        let total_height = if let Some(ref button) = self.more_button {
            button.rect.bottom()
        } else if let Some(bubble) = self.bubbles.last() {
            bubble.rect().bottom()
        } else {
            0
        };
        self.scrollbar.set_total_height(total_height);
        self.scrollbar.scroll_to(total_height);
    }

    fn rebuild_bubbles(&mut self, profile: &Profile, conv: Conversation) {
        debug_log!("Rebuilding conversation bubbles");
        self.conv = conv;
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
    }
}

//===========================================================================//

struct MoreButton {
    rect: Rect<i32>,
    hovering: bool,
}

impl MoreButton {
    fn new(width: i32, top: i32) -> MoreButton {
        MoreButton {
            rect: Rect::new(0, top, width, MORE_BUTTON_HEIGHT),
            hovering: false,
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = if self.hovering {
            (1.0, 0.5, 0.1)
        } else {
            (0.5, 0.25, 0.1)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       BUBBLE_FONT_SIZE,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       "- More -");
    }

    fn on_event(&mut self, event: &Event) -> bool {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return true;
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
            _ => {}
        }
        return false;
    }
}

//===========================================================================//

trait BubbleView {
    fn rect(&self) -> Rect<i32>;

    fn is_choice_or_puzzle(&self) -> bool { false }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>);

    fn on_event(&mut self, _event: &Event) -> Option<ConverseAction> { None }
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

    fn is_choice_or_puzzle(&self) -> bool { true }

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

    fn on_event(&mut self, event: &Event) -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return Some(ConverseAction::GoToPuzzle(self.puzzle));
                }
            }
            Event::MouseMove(mouse) => {
                self.hovering = self.rect.contains_point(mouse.pt);
            }
            Event::Unfocus => self.hovering = false,
            _ => {}
        }
        return None;
    }
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

    fn is_choice_or_puzzle(&self) -> bool { true }

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

    fn on_event(&mut self, event: &Event) -> Option<ConverseAction> {
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
            Event::Unfocus => self.hovering = None,
            _ => {}
        }
        return None;
    }
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
