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
    BubbleAction, BubbleKind, BubbleView, CutsceneBubbleView,
    PuzzleBubbleView, ReachedAction, SpeechBubbleView, YouChoiceBubbleView,
};
use crate::mancer::font::Align;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::{Event, Keycode, Resources, Ui};
use crate::mancer::save::{Prefs, Profile};
use crate::mancer::state::{
    ConversationBubble, ConversationExt, Cutscene, GameState,
};
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

pub enum SequenceAction {
    GoToPuzzle(Puzzle),
    ConversationCompleted,
    PlayCutscene(Cutscene),
    UnlockPuzzles(Vec<Puzzle>),
}

//===========================================================================//

pub struct BubbleSequenceView {
    rect: Rect<i32>,
    conv: Conversation,
    bubbles: Vec<Box<dyn BubbleView>>,
    bubbles_are_complete: bool,
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
            bubbles_are_complete: false,
            num_bubbles_shown: 0,
            more_button: None,
            scrollbar: Scrollbar::new(scrollbar_rect, 0),
        };
        view.reset(ui, state, None);
        view
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Define clipping area:
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
        state: &mut GameState,
    ) -> Option<SequenceAction> {
        // Handle scrollbar events:
        self.scrollbar.on_event(event, ui);
        match event {
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y, ui);
            }
            _ => {}
        }

        // Handle more-button events:
        let bubble_event = event.relative_to(
            self.rect.top_left() - vec2(0, self.scrollbar.scroll_top()),
        );
        let mut should_advance = false;
        if let Some(ref mut button) = self.more_button {
            if button.on_event(&bubble_event, ui) {
                should_advance = true;
            }
        }

        // Handle conversation bubble events:
        for bubble in self.bubbles.iter_mut().take(self.num_bubbles_shown) {
            match bubble.on_event(&bubble_event, ui) {
                Some(BubbleAction::GoToPuzzle(puzzle)) => {
                    return Some(SequenceAction::GoToPuzzle(puzzle));
                }
                Some(BubbleAction::MakeChoice(key, value)) => {
                    state.set_current_conversation_choice(key, value);
                    should_advance = true;
                    break;
                }
                Some(BubbleAction::ParagraphFinished) => {
                    state.set_current_conversation_progress(
                        self.num_bubbles_shown,
                    );
                    let mut should_pause = bubble.should_pause_afterwards();
                    if let Some(next) =
                        self.bubbles.get(self.num_bubbles_shown)
                    {
                        if next.kind() == BubbleKind::Choice {
                            should_pause = false;
                        }
                    } else if self.bubbles_are_complete {
                        should_pause = false;
                    }
                    if should_pause {
                        debug_assert!(self.more_button.is_some());
                        if let Some(ref mut button) = self.more_button {
                            button.set_visible(ui, true);
                        }
                    } else {
                        should_advance = true;
                    }
                    break;
                }
                Some(BubbleAction::PlayCutscene(cutscene)) => {
                    return Some(SequenceAction::PlayCutscene(cutscene));
                }
                None => {}
            }
        }

        if should_advance {
            if self.num_bubbles_shown >= self.bubbles.len()
                && self.bubbles_are_complete
            {
                state.mark_current_conversation_complete();
                let action = self.advance(ui, state);
                debug_assert!(action.is_none());
                return Some(SequenceAction::ConversationCompleted);
            } else {
                state
                    .set_current_conversation_progress(self.num_bubbles_shown);
                return self.advance(ui, state);
            }
        }
        return None;
    }

    pub fn reset(
        &mut self,
        ui: &mut Ui,
        state: &GameState,
        jump_to_puzzle: Option<Puzzle>,
    ) {
        // Rebuild bubbles and set relevant fields:
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        self.rebuild_bubbles(
            profile,
            state.prefs(),
            profile.current_conversation(),
        );
        self.num_bubbles_shown =
            profile.conversation_progress(self.conv).min(self.bubbles.len());
        ui.request_redraw();

        // Skip paragraphs for already-completed speech bubbles:
        for bubble in self.bubbles.iter_mut().take(self.num_bubbles_shown) {
            bubble.skip_paragraph(ui);
        }

        // Determine if we should also show the next bubble, and whether we
        // need to display the "More" button yet:
        let next_bubble_kind =
            self.bubbles.get(self.num_bubbles_shown).map(|b| b.kind());
        let (show_next, more_button_visible) = match next_bubble_kind {
            None | Some(BubbleKind::Cutscene) | Some(BubbleKind::Puzzle) => {
                (false, true)
            }
            Some(BubbleKind::Choice) => (true, true),
            Some(BubbleKind::Speech) => {
                let show_next = if self.num_bubbles_shown == 0 {
                    true
                } else {
                    match self.bubbles[self.num_bubbles_shown - 1].kind() {
                        BubbleKind::Choice | BubbleKind::Speech => false,
                        BubbleKind::Cutscene | BubbleKind::Puzzle => true,
                    }
                };
                (show_next, !show_next)
            }
        };
        if show_next {
            self.num_bubbles_shown += 1;
            ui.request_redraw();
        }
        self.add_more_button_if_needed(more_button_visible);

        // Jump scrollbar:
        self.update_scrollbar_height(ui);
        if let Some(puzzle) = jump_to_puzzle {
            let mut position = 0;
            for bubble in self.bubbles.iter() {
                if bubble.has_puzzle(puzzle) {
                    let rect = bubble.rect();
                    position = rect.y + rect.height / 2;
                    break;
                }
            }
            self.scrollbar.scroll_to(position, ui);
        } else if profile.is_conversation_complete(self.conv) {
            self.scrollbar.scroll_to(0, ui);
        } else {
            self.scrollbar.scroll_to(self.total_height(), ui);
        }
    }

    fn advance(
        &mut self,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<SequenceAction> {
        // Increment self.num_bubbles_shown, rebuliding bubbles if necessary:
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        debug_assert_eq!(profile.current_conversation(), self.conv);
        let progress = profile.conversation_progress(self.conv);
        debug_assert!(
            profile.is_conversation_complete(self.conv)
                || progress == self.num_bubbles_shown
        );
        if self.num_bubbles_shown == self.bubbles.len()
            && !self.bubbles_are_complete
        {
            self.rebuild_bubbles(profile, state.prefs(), self.conv);
            for bubble in self.bubbles.iter_mut().take(progress) {
                bubble.skip_paragraph(ui);
            }
        }
        let next_bubble_index = self.num_bubbles_shown;
        self.num_bubbles_shown =
            (self.num_bubbles_shown + 1).min(self.bubbles.len());
        ui.request_redraw();

        // Determine whether we need to display the "More" button yet:
        if let Some(next_bubble) = self.bubbles.get(next_bubble_index) {
            let visible = match next_bubble.kind() {
                BubbleKind::Choice | BubbleKind::Speech => false,
                BubbleKind::Cutscene | BubbleKind::Puzzle => true,
            };
            self.add_more_button_if_needed(visible);
        }

        // Ease scrollbar to the bottom:
        self.update_scrollbar_height(ui);
        self.scrollbar.ease_to(self.total_height());

        // Return the on-reached action for the newly-revealed bubble, if any:
        if let Some(next_bubble) = self.bubbles.get(next_bubble_index) {
            if let Some(action) = next_bubble.on_first_reached() {
                state
                    .set_current_conversation_progress(self.num_bubbles_shown);
                match action {
                    ReachedAction::PlayCutscene(cutscene) => {
                        return Some(SequenceAction::PlayCutscene(cutscene));
                    }
                    ReachedAction::UnlockPuzzles(puzzles) => {
                        return Some(SequenceAction::UnlockPuzzles(puzzles));
                    }
                }
            }
        }
        return None;
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
        let (bubbles, is_complete) = conv.generate_bubbles(profile);
        self.bubbles_are_complete = is_complete;
        let num_bubbles = bubbles.len();
        self.bubbles = Vec::<Box<dyn BubbleView>>::with_capacity(num_bubbles);
        for (bubble_index, bubble) in bubbles.into_iter().enumerate() {
            let bubble_top = if let Some(last) = self.bubbles.last() {
                last.rect().bottom() + BUBBLE_SPACING
            } else {
                0
            };
            let is_last = is_complete && bubble_index + 1 == num_bubbles;
            let bubble_view = match bubble {
                ConversationBubble::Cutscene(cutscene) => {
                    CutsceneBubbleView::new(bubble_width, bubble_top, cutscene)
                }
                ConversationBubble::NpcSpeech(portrait, format, interrupt) => {
                    SpeechBubbleView::new(
                        bubble_width,
                        bubble_top,
                        Some(portrait),
                        prefs,
                        &format,
                        !interrupt && !is_last,
                    )
                }
                ConversationBubble::Puzzles(puzzles) => {
                    PuzzleBubbleView::new(bubble_width, bubble_top, puzzles)
                }
                ConversationBubble::YouChoice(key, choices) => {
                    YouChoiceBubbleView::new(
                        bubble_width,
                        bubble_top,
                        prefs,
                        key,
                        choices,
                    )
                }
                ConversationBubble::YouSpeech(format) => {
                    SpeechBubbleView::new(
                        bubble_width,
                        bubble_top,
                        None,
                        prefs,
                        &format,
                        !is_last,
                    )
                }
            };
            self.bubbles.push(bubble_view);
        }
    }

    fn add_more_button_if_needed(&mut self, visible: bool) {
        self.more_button = if self.num_bubbles_shown < self.bubbles.len() {
            let width = self.rect.width - (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
            let top = if self.num_bubbles_shown == 0 {
                0
            } else {
                self.bubbles[self.num_bubbles_shown - 1].rect().bottom()
                    + BUBBLE_SPACING
            };
            Some(MoreButton::new(width, top, visible))
        } else {
            None
        };
    }

    fn update_scrollbar_height(&mut self, ui: &mut Ui) {
        self.scrollbar.set_total_height(self.total_height(), ui);
    }

    fn total_height(&self) -> i32 {
        if let Some(ref button) = self.more_button {
            button.rect.bottom()
        } else if let Some(bubble) = self.bubbles.last() {
            bubble.rect().bottom()
        } else {
            0
        }
    }
}

//===========================================================================//

struct MoreButton {
    rect: Rect<i32>,
    hovering: bool,
    visible: bool,
}

impl MoreButton {
    fn new(width: i32, top: i32, visible: bool) -> MoreButton {
        MoreButton {
            rect: Rect::new(0, top, width, MORE_BUTTON_HEIGHT),
            hovering: false,
            visible,
        }
    }

    fn set_visible(&mut self, ui: &mut Ui, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            if !visible {
                self.hovering = false;
            }
            ui.request_redraw();
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        if !self.visible {
            return;
        }
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
        if !self.visible {
            return false;
        }
        match event {
            Event::KeyDown(key) if key.code == Keycode::Return => {
                return true;
            }
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
