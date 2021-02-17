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

mod bubble;
mod sequence;

use self::sequence::{BubbleSequenceView, SequenceAction};
use super::list::{list_height_for_num_items, ListIcon, ListView};
use crate::mancer::gui::{Event, Resources, Ui};
use crate::mancer::state::{Cutscene, GameState};
use cgmath::Matrix4;
use tachy::geom::Rect;
use tachy::save::{Chapter, Conversation, Puzzle};

//===========================================================================//

const CHAPTER_LIST_WIDTH: i32 = 120;
const CHAPTER_LIST_HEIGHT: i32 = list_height_for_num_items(6);
const CONV_LIST_WIDTH: i32 = 220;
const LIST_MARGIN_HORZ: i32 = 22;

//===========================================================================//

pub enum ConverseAction {
    GoToPuzzle(Puzzle),
    PlayCutscene(Cutscene),
    UnlockPuzzles(Vec<Puzzle>),
}

//===========================================================================//

pub struct ConverseView {
    chapter_list: ListView<Chapter>,
    conv_list: ListView<Conversation>,
    bubble_seq: BubbleSequenceView,
}

impl ConverseView {
    pub fn new(
        rect: Rect<i32>,
        ui: &mut Ui,
        state: &GameState,
    ) -> ConverseView {
        let chapter_list_left = rect.x;
        let chapter_list_top =
            rect.y + (rect.height - CHAPTER_LIST_HEIGHT) / 2;
        let conv_list_left =
            chapter_list_left + CHAPTER_LIST_WIDTH + LIST_MARGIN_HORZ;
        let bubble_seq_left =
            conv_list_left + CONV_LIST_WIDTH + LIST_MARGIN_HORZ;
        let bubble_seq_width = rect.right() - bubble_seq_left;

        let conversation = state.current_conversation();
        ConverseView {
            chapter_list: ListView::new(
                Rect::new(
                    chapter_list_left,
                    chapter_list_top,
                    CHAPTER_LIST_WIDTH,
                    CHAPTER_LIST_HEIGHT,
                ),
                ui,
                chapter_list_items(state),
                &conversation.chapter(),
            ),
            conv_list: ListView::new(
                Rect::new(
                    conv_list_left,
                    rect.y,
                    CONV_LIST_WIDTH,
                    rect.height,
                ),
                ui,
                conv_list_items(state),
                &conversation,
            ),
            bubble_seq: BubbleSequenceView::new(
                Rect::new(
                    bubble_seq_left,
                    rect.y,
                    bubble_seq_width,
                    rect.height,
                ),
                ui,
                state,
            ),
        }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        let conv = state.current_conversation();
        self.chapter_list.draw(resources, matrix, &conv.chapter());
        self.conv_list.draw(resources, matrix, &conv);
        self.bubble_seq.draw(resources, matrix);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<ConverseAction> {
        // Debug events:
        match event {
            Event::Debug(key, _) if key == "ResetConv" => {
                state.reset_current_conversation_progress();
                self.update_chapters(ui, state);
                self.update_conv_list(ui, state);
                self.bubble_seq.reset(ui, state, None);
            }
            _ => {}
        }

        // Conversations list:
        if let Some(conv) =
            self.conv_list.on_event(event, ui, &state.current_conversation())
        {
            state.set_current_conversation(conv);
            ui.request_redraw();
            self.bubble_seq.reset(ui, state, None);
            return None;
        }

        // Chapters list:
        if let Some(chapter) = self.chapter_list.on_event(
            event,
            ui,
            &state.current_conversation().chapter(),
        ) {
            let conv = Conversation::all()
                .find(|&conv| {
                    conv.chapter() == chapter
                        && state.is_conversation_unlocked(conv)
                })
                .unwrap_or(Conversation::first());
            state.set_current_conversation(conv);
            ui.request_redraw();
            self.update_conv_list(ui, state);
            self.bubble_seq.reset(ui, state, None);
            return None;
        }

        // Bubble sequence:
        match self.bubble_seq.on_event(event, ui, state) {
            Some(SequenceAction::GoToPuzzle(puzzle)) => {
                return Some(ConverseAction::GoToPuzzle(puzzle));
            }
            Some(SequenceAction::ConversationCompleted) => {
                self.update_chapters(ui, state);
                self.update_conv_list(ui, state);
            }
            Some(SequenceAction::PlayCutscene(cutscene)) => {
                return Some(ConverseAction::PlayCutscene(cutscene));
            }
            Some(SequenceAction::UnlockPuzzles(puzzles)) => {
                return Some(ConverseAction::UnlockPuzzles(puzzles));
            }
            None => {}
        }
        return None;
    }

    pub fn jump_to_current_conversation_from(
        &mut self,
        puzzle: Puzzle,
        ui: &mut Ui,
        state: &GameState,
    ) {
        self.update_chapters(ui, state);
        self.update_conv_list(ui, state);
        self.bubble_seq.reset(ui, state, Some(puzzle));
    }

    fn update_chapters(&mut self, ui: &mut Ui, state: &GameState) {
        self.chapter_list.set_items(
            ui,
            chapter_list_items(state),
            &state.current_conversation().chapter(),
        );
    }

    fn update_conv_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.conv_list.set_items(
            ui,
            conv_list_items(state),
            &state.current_conversation(),
        );
    }
}

fn chapter_list_items(
    state: &GameState,
) -> Vec<(Chapter, String, bool, Option<ListIcon>)> {
    state
        .unlocked_chapters()
        .into_iter()
        .map(|chapter| (chapter, chapter.title().to_string(), false, None))
        .collect()
}

fn conv_list_items(
    state: &GameState,
) -> Vec<(Conversation, String, bool, Option<ListIcon>)> {
    let chapter = state.current_conversation().chapter();
    Conversation::all()
        .filter(|&conv| {
            conv.chapter() == chapter && state.is_conversation_unlocked(conv)
        })
        .map(|conv| {
            let label = conv.title().to_string();
            (conv, label, !state.is_conversation_complete(conv), None)
        })
        .collect()
}

//===========================================================================//
