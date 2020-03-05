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

use super::super::super::button::Scrollbar;
use super::bubble::{
    BubbleAction, BubbleView, CutsceneBubbleView, NpcSpeechBubbleView,
    PuzzleBubbleView, YouChoiceBubbleView, YouSpeechBubbleView,
};
use crate::mancer::font::Align;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::{Event, Resources, Ui};
use crate::mancer::save::{Prefs, Profile};
use crate::mancer::state::{ConversationBubble, ConversationExt, GameState};
use cgmath::{vec2, Matrix4};
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect};
use tachy::save::{Conversation, Puzzle};

//===========================================================================//

const BUBBLE_SPACING: i32 = 16;

const MORE_BUTTON_FONT_SIZE: f32 = 20.0;
const MORE_BUTTON_HEIGHT: i32 = 30;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 8;

//===========================================================================//

#[derive(Clone, Copy)]
pub enum BubblesScroll {
    JumpToTopOrBottom,
    JumpToPuzzle(Puzzle),
    EaseToBottom,
}

//===========================================================================//

pub struct BubbleSequenceView {
    rect: Rect<i32>,
    conv: Conversation,
    bubbles: Vec<Box<dyn BubbleView>>,
    num_bubbles_shown: usize,
    more_button: Option<MoreButton>,
    scrollbar: Scrollbar,
}

impl BubbleSequenceView {
    pub fn new(
        rect: Rect<i32>,
        ui: &mut Ui,
        state: &GameState,
    ) -> BubbleSequenceView {
        let scrollbar_rect = Rect::new(
            rect.right() - SCROLLBAR_WIDTH,
            rect.y,
            SCROLLBAR_WIDTH,
            rect.height,
        );
        let mut view = BubbleSequenceView {
            rect,
            conv: Conversation::first(),
            bubbles: Vec::new(),
            num_bubbles_shown: 0,
            more_button: None,
            scrollbar: Scrollbar::new(scrollbar_rect, 0),
        };
        view.update_conversation(BubblesScroll::JumpToTopOrBottom, ui, state);
        view
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw background and define clipping area:
        let stencil = Stencil::new();
        {
            resources.shaders().solid().tint_rect(
                &matrix,
                Color4::TRANSPARENT,
                self.rect.as_f32(),
            );
        }
        stencil.enable_clipping();

        // Draw conversation bubbles:
        let scroll_top = self.scrollbar.scroll_top();
        let bubble_matrix = matrix
            * Matrix4::trans2(
                self.rect.x as f32,
                (self.rect.y - scroll_top) as f32,
            );
        for bubble in self.bubbles.iter().take(self.num_bubbles_shown) {
            let rect = bubble.rect();
            if rect.bottom() > scroll_top
                && rect.y < scroll_top + self.rect.height
            {
                bubble.draw(resources, &bubble_matrix);
            }
        }
        if let Some(ref button) = self.more_button {
            let rect = button.rect;
            if rect.bottom() > scroll_top
                && rect.y < scroll_top + self.rect.height
            {
                button.draw(resources, &bubble_matrix);
            }
        }

        // Draw scrollbar:
        self.scrollbar.draw(resources, matrix);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubbleAction> {
        // Handle scrollbar events:
        self.scrollbar.on_event(event, ui);
        match event {
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y, ui);
            }
            _ => {}
        }

        // Handle conversation bubble events:
        let bubble_event = event.relative_to(
            self.rect.top_left() - vec2(0, self.scrollbar.scroll_top()),
        );
        for bubble in self.bubbles.iter_mut().take(self.num_bubbles_shown) {
            if let Some(action) = bubble.on_event(&bubble_event, ui) {
                return Some(action);
            }
        }
        if let Some(ref mut button) = self.more_button {
            if button.on_event(&bubble_event, ui) {
                if self.num_bubbles_shown + 1 < self.bubbles.len() {
                    return Some(BubbleAction::Increment);
                }
                if let Some(ref bubble) = self.bubbles.last() {
                    if bubble.is_choice_or_puzzle() {
                        return Some(BubbleAction::Increment);
                    }
                }
                return Some(BubbleAction::Complete);
            }
        }
        return None;
    }

    pub fn reset(&mut self, ui: &mut Ui, state: &GameState) {
        self.bubbles.clear();
        self.num_bubbles_shown = 0;
        self.more_button = None;
        ui.request_redraw();
        self.update_conversation(BubblesScroll::JumpToTopOrBottom, ui, state);
    }

    pub fn update_conversation(
        &mut self,
        scroll: BubblesScroll,
        ui: &mut Ui,
        state: &GameState,
    ) {
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        let conv = profile.current_conversation();
        let num_bubbles_completed = profile.conversation_progress(conv);
        let num_bubbles_shown = num_bubbles_completed.saturating_add(1);
        if conv != self.conv || num_bubbles_shown > self.bubbles.len() {
            self.rebuild_bubbles(profile, state.prefs(), conv);
        }
        self.num_bubbles_shown = num_bubbles_shown.min(self.bubbles.len());
        ui.request_redraw();
        for bubble in self.bubbles.iter_mut().take(num_bubbles_completed) {
            bubble.on_event(&Event::Unfocus, ui);
        }

        self.more_button = if self.num_bubbles_shown < self.bubbles.len() {
            let width = self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
            let top = if self.num_bubbles_shown == 0 {
                0
            } else {
                self.bubbles[self.num_bubbles_shown - 1].rect().bottom()
                    + BUBBLE_SPACING
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
        self.scrollbar.set_total_height(total_height, ui);
        match scroll {
            BubblesScroll::JumpToTopOrBottom => {
                if profile.is_conversation_complete(conv) {
                    self.scrollbar.scroll_to(0, ui);
                } else {
                    self.scrollbar.scroll_to(total_height, ui);
                }
            }
            BubblesScroll::JumpToPuzzle(puzzle) => {
                let mut position = 0;
                for bubble in self.bubbles.iter() {
                    if bubble.has_puzzle(puzzle) {
                        let rect = bubble.rect();
                        position = rect.y + rect.height / 2;
                        break;
                    }
                }
                self.scrollbar.scroll_to(position, ui);
            }
            BubblesScroll::EaseToBottom => {
                self.scrollbar.ease_to(total_height);
            }
        }
    }

    fn rebuild_bubbles(
        &mut self,
        profile: &Profile,
        prefs: &Prefs,
        conv: Conversation,
    ) {
        debug_log!("Rebuilding conversation bubbles");
        self.conv = conv;
        let bubble_width =
            self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
        let mut bubble_views = Vec::<Box<dyn BubbleView>>::new();
        for bubble in conv.bubbles(profile) {
            let bubble_top = if let Some(last) = bubble_views.last() {
                last.rect().bottom() + BUBBLE_SPACING
            } else {
                0
            };
            match bubble {
                ConversationBubble::Cutscene(cutscene) => {
                    bubble_views.push(CutsceneBubbleView::new(
                        bubble_width,
                        bubble_top,
                        cutscene,
                    ))
                }
                ConversationBubble::NpcSpeech(portrait, format) => {
                    bubble_views.push(NpcSpeechBubbleView::new(
                        bubble_width,
                        bubble_top,
                        portrait,
                        prefs,
                        &format,
                    ))
                }
                ConversationBubble::Puzzles(puzzles) => {
                    bubble_views.push(PuzzleBubbleView::new(
                        bubble_width,
                        bubble_top,
                        puzzles,
                    ));
                }
                ConversationBubble::YouChoice(key, choices) => bubble_views
                    .push(YouChoiceBubbleView::new(
                        bubble_width,
                        bubble_top,
                        key,
                        choices,
                    )),
                ConversationBubble::YouSpeech(format) => {
                    bubble_views.push(YouSpeechBubbleView::new(
                        bubble_width,
                        bubble_top,
                        prefs,
                        &format,
                    ))
                }
            }
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
            Color3::new(1.0, 0.5, 0.1)
        } else {
            Color3::new(0.5, 0.25, 0.1)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(
            &matrix,
            MORE_BUTTON_FONT_SIZE,
            Align::MidCenter,
            (rect.x + 0.5 * rect.width, rect.y + 0.5 * rect.height),
            "- More -",
        );
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui) -> bool {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return true;
                }
            }
            Event::MouseMove(mouse) => {
                let hovering = self.rect.contains_point(mouse.pt);
                if self.hovering != hovering {
                    self.hovering = hovering;
                    ui.request_redraw();
                }
            }
            Event::Unfocus => {
                if self.hovering {
                    self.hovering = false;
                    ui.request_redraw();
                }
            }
            _ => {}
        }
        return false;
    }
}

//===========================================================================//
