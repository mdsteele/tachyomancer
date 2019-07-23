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

use super::list::{ListIcon, ListView, list_height_for_num_items};
use super::super::button::Scrollbar;
use super::super::paragraph::Paragraph;
use cgmath::{Matrix4, Point2, vec2};
use num_integer::div_mod_floor;
use std::collections::HashSet;
use tachy::font::Align;
use tachy::geom::{AsFloat, Color3, MatrixExt, Rect};
use tachy::gl::Stencil;
use tachy::gui::{Event, Resources, Ui};
use tachy::save::{Chapter, Conversation, Prefs, Profile, Puzzle};
use tachy::state::{ConversationBubble, ConversationExt, Cutscene, GameState,
                   Portrait};

//===========================================================================//

const CHAPTER_LIST_WIDTH: i32 = 120;
const CHAPTER_LIST_HEIGHT: i32 = list_height_for_num_items(5);
const CONV_LIST_WIDTH: i32 = 220;
const ELEMENT_SPACING: i32 = 22;

const BUBBLE_FONT_SIZE: f32 = 20.0;
const BUBBLE_INNER_MARGIN: i32 = 12;
const BUBBLE_LINE_HEIGHT: f32 = 22.0;
const BUBBLE_SPACING: i32 = 16;

const CHOICE_HEIGHT: i32 = 30;
const CHOICE_SPACING: i32 = 2;

const CUTSCENE_BUBBLE_HEIGHT: i32 = 50;

const MORE_BUTTON_HEIGHT: i32 = 30;

const PORTRAIT_HEIGHT: i32 = 85;
const PORTRAIT_WIDTH: i32 = 68;

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
    PlayCutscene(Cutscene),
}

//===========================================================================//

pub struct ConverseView {
    chapter_list: ListView<Chapter>,
    conv_list: ListView<Conversation>,
    bubbles_list: BubblesListView,
}

impl ConverseView {
    pub fn new(rect: Rect<i32>, ui: &mut Ui, state: &GameState)
               -> ConverseView {
        let chapter_list_left = rect.x;
        let chapter_list_top = rect.y +
            (rect.height - CHAPTER_LIST_HEIGHT) / 2;
        let conv_list_left = chapter_list_left + CHAPTER_LIST_WIDTH +
            ELEMENT_SPACING;
        let bubbles_list_left = conv_list_left + CONV_LIST_WIDTH +
            ELEMENT_SPACING;
        let bubbles_list_width = rect.right() - bubbles_list_left;

        let conversation = state.current_conversation();
        ConverseView {
            chapter_list: ListView::new(Rect::new(chapter_list_left,
                                                  chapter_list_top,
                                                  CHAPTER_LIST_WIDTH,
                                                  CHAPTER_LIST_HEIGHT),
                                        ui,
                                        chapter_list_items(state),
                                        &conversation.chapter()),
            conv_list: ListView::new(Rect::new(conv_list_left,
                                               rect.y,
                                               CONV_LIST_WIDTH,
                                               rect.height),
                                     ui,
                                     conv_list_items(state),
                                     &conversation),
            bubbles_list: BubblesListView::new(Rect::new(bubbles_list_left,
                                                         rect.y,
                                                         bubbles_list_width,
                                                         rect.height),
                                               ui,
                                               state),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        let conv = state.current_conversation();
        self.chapter_list.draw(resources, matrix, &conv.chapter());
        self.conv_list.draw(resources, matrix, &conv);
        self.bubbles_list.draw(resources, matrix);
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui,
                    state: &mut GameState)
                    -> Option<ConverseAction> {
        match event {
            Event::Debug(key, _) if key == "resetconv" => {
                state.reset_current_conversation_progress();
                self.bubbles_list.reset(ui, state);
            }
            _ => {}
        }
        if let Some(conv) =
            self.conv_list.on_event(event, ui, &state.current_conversation())
        {
            state.set_current_conversation(conv);
            ui.request_redraw();
            self.update_conversation_bubbles(ui, state);
            None
        } else if let Some(chapter) =
            self.chapter_list
                .on_event(event, ui, &state.current_conversation().chapter())
        {
            let conv = Conversation::all()
                .find(|&conv| {
                          conv.chapter() == chapter &&
                              state.is_conversation_unlocked(conv)
                      })
                .unwrap_or(Conversation::first());

            state.set_current_conversation(conv);
            ui.request_redraw();
            self.update_conversation_list(ui, state);
            self.update_conversation_bubbles(ui, state);
            None
        } else {
            self.bubbles_list.on_event(event, ui)
        }
    }

    pub fn update_conversation_bubbles(&mut self, ui: &mut Ui,
                                       state: &GameState) {
        self.bubbles_list.update_conversation(ui, state);
    }

    pub fn update_conversation_list(&mut self, ui: &mut Ui,
                                    state: &GameState) {
        self.conv_list.set_items(ui,
                                 conv_list_items(state),
                                 &state.current_conversation());
    }
}

fn chapter_list_items(state: &GameState)
                      -> Vec<(Chapter, String, Option<ListIcon>)> {
    let chapters: HashSet<Chapter> = Conversation::all()
        .filter(|&conv| state.is_conversation_unlocked(conv))
        .map(|conv| conv.chapter())
        .collect();
    state
        .chapter_order()
        .into_iter()
        .filter(|chapter| chapters.contains(chapter))
        .map(|chapter| (chapter, chapter.title().to_string(), None))
        .collect()
}

fn conv_list_items(state: &GameState)
                   -> Vec<(Conversation, String, Option<ListIcon>)> {
    let chapter = state.current_conversation().chapter();
    Conversation::all()
        .filter(|&conv| {
                    conv.chapter() == chapter &&
                        state.is_conversation_unlocked(conv)
                })
        .map(|conv| {
                 let mut label = conv.title().to_string();
                 if !state.is_conversation_complete(conv) {
                     label = format!("* {}", label);
                 }
                 (conv, label, None)
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
    fn new(rect: Rect<i32>, ui: &mut Ui, state: &GameState)
           -> BubblesListView {
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
            scrollbar: Scrollbar::new(scrollbar_rect, 0),
        };
        view.update_conversation(ui, state);
        view
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw background and define clipping area:
        let stencil = Stencil::new();
        {
            let color = Color3::new(0.1, 0.1, 0.1);
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

    fn on_event(&mut self, event: &Event, ui: &mut Ui)
                -> Option<ConverseAction> {
        // Handle scrollbar events:
        self.scrollbar.on_event(event, ui);
        match event {
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y, ui);
            }
            _ => {}
        }

        // Handle conversation bubble events:
        let bubble_event =
            event.relative_to(self.rect.top_left() -
                                  vec2(0, self.scrollbar.scroll_top()));
        for bubble in self.bubbles.iter_mut().take(self.num_bubbles_shown) {
            if let Some(action) = bubble.on_event(&bubble_event, ui) {
                return Some(action);
            }
        }
        if let Some(ref mut button) = self.more_button {
            if button.on_event(&bubble_event, ui) {
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

    fn reset(&mut self, ui: &mut Ui, state: &GameState) {
        self.bubbles.clear();
        self.num_bubbles_shown = 0;
        self.more_button = None;
        ui.request_redraw();
        self.update_conversation(ui, state);
    }

    fn update_conversation(&mut self, ui: &mut Ui, state: &GameState) {
        debug_assert!(state.profile().is_some());
        let profile = state.profile().unwrap();
        let conv = profile.current_conversation();
        let num_bubbles_shown =
            profile.conversation_progress(conv).saturating_add(1);
        if conv != self.conv || num_bubbles_shown > self.bubbles.len() {
            self.rebuild_bubbles(profile, state.prefs(), conv);
        }
        self.num_bubbles_shown = num_bubbles_shown.min(self.bubbles.len());
        ui.request_redraw();

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
        self.scrollbar.set_total_height(total_height, ui);
        self.scrollbar.scroll_to(total_height, ui);
    }

    fn rebuild_bubbles(&mut self, profile: &Profile, prefs: &Prefs,
                       conv: Conversation) {
        debug_log!("Rebuilding conversation bubbles");
        self.conv = conv;
        let bubble_width = self.rect.width -
            (SCROLLBAR_MARGIN + SCROLLBAR_WIDTH);
        let mut bubble_top: i32 = 0;
        let bubble_seq = conv.bubbles(profile);
        let mut bubble_views = Vec::with_capacity(bubble_seq.len());
        for bubble in bubble_seq {
            if bubble_top > 0 {
                bubble_top += BUBBLE_SPACING;
            }
            let bubble_view = match bubble {
                ConversationBubble::Cutscene(cutscene) => {
                    CutsceneBubbleView::new(bubble_width, bubble_top, cutscene)
                }
                ConversationBubble::NpcSpeech(portrait, format) => {
                    NpcSpeechBubbleView::new(bubble_width,
                                             bubble_top,
                                             portrait,
                                             prefs,
                                             &format)
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
                ConversationBubble::YouSpeech(format) => {
                    YouSpeechBubbleView::new(bubble_width,
                                             bubble_top,
                                             prefs,
                                             &format)
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
            Color3::new(1.0, 0.5, 0.1)
        } else {
            Color3::new(0.5, 0.25, 0.1)
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

trait BubbleView {
    fn rect(&self) -> Rect<i32>;

    fn is_choice_or_puzzle(&self) -> bool { false }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>);

    fn on_event(&mut self, _event: &Event, _ui: &mut Ui)
                -> Option<ConverseAction> {
        None
    }
}

//===========================================================================//

struct CutsceneBubbleView {
    rect: Rect<i32>,
    cutscene: Cutscene,
    hovering: bool,
}

impl CutsceneBubbleView {
    fn new(width: i32, top: i32, cutscene: Cutscene) -> Box<BubbleView> {
        let view = CutsceneBubbleView {
            rect: Rect::new(0, top, width, CUTSCENE_BUBBLE_HEIGHT),
            cutscene,
            hovering: false,
        };
        Box::new(view)
    }
}

impl BubbleView for CutsceneBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = if self.hovering {
            Color3::new(0.1, 1.0, 1.0)
        } else {
            Color3::new(0.1, 0.5, 0.5)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(&matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       BUBBLE_FONT_SIZE,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       "Replay cutscene");
    }

    fn on_event(&mut self, event: &Event, ui: &mut Ui)
                -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return Some(ConverseAction::PlayCutscene(self.cutscene));
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
        return None;
    }
}

//===========================================================================//

struct NpcSpeechBubbleView {
    rect: Rect<i32>,
    portrait: Portrait,
    paragraph: Paragraph,
}

impl NpcSpeechBubbleView {
    fn new(width: i32, top: i32, portrait: Portrait, prefs: &Prefs,
           format: &str)
           -> Box<BubbleView> {
        let wrap_width = width - PORTRAIT_WIDTH - 3 * BUBBLE_INNER_MARGIN;
        let paragraph = Paragraph::compile(BUBBLE_FONT_SIZE,
                                           BUBBLE_LINE_HEIGHT,
                                           wrap_width as f32,
                                           prefs,
                                           format);
        let height =
            (PORTRAIT_HEIGHT + 2 * BUBBLE_INNER_MARGIN)
                .max(2 * BUBBLE_INNER_MARGIN +
                         (paragraph.height().ceil() as i32));
        let view = NpcSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            portrait,
            paragraph,
        };
        Box::new(view)
    }
}

impl BubbleView for NpcSpeechBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        // Draw bubble:
        let rect = self.rect.as_f32();
        let color = Color3::new(0.1, 0.5, 0.1);
        resources.shaders().solid().fill_rect(matrix, color, rect);

        // Draw portrait:
        let portrait_left_top = Point2::new(self.rect.x + BUBBLE_INNER_MARGIN,
                                            self.rect.y + BUBBLE_INNER_MARGIN);
        resources.textures().portraits().bind();
        resources
            .shaders()
            .portrait()
            .draw(matrix, self.portrait as u32, portrait_left_top.as_f32());

        // Draw paragraph:
        let left = (self.rect.x + PORTRAIT_WIDTH +
            2 * BUBBLE_INNER_MARGIN) as f32;
        let top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));
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
            Color3::new(0.1, 1.0, 1.0)
        } else {
            Color3::new(0.1, 0.5, 0.5)
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

    fn on_event(&mut self, event: &Event, ui: &mut Ui)
                -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    return Some(ConverseAction::GoToPuzzle(self.puzzle));
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
                Color3::new(1.0, 1.0, 0.1)
            } else {
                Color3::new(0.5, 0.5, 0.1)
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

    fn on_event(&mut self, event: &Event, ui: &mut Ui)
                -> Option<ConverseAction> {
        match event {
            Event::MouseDown(mouse) => {
                if let Some(index) = self.choice_for_pt(mouse.pt) {
                    let key = self.key.clone();
                    let value = self.choices[index].0.clone();
                    return Some(ConverseAction::MakeChoice(key, value));
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

struct YouSpeechBubbleView {
    rect: Rect<i32>,
    paragraph: Paragraph,
}

impl YouSpeechBubbleView {
    fn new(width: i32, top: i32, prefs: &Prefs, format: &str)
           -> Box<BubbleView> {
        let wrap_width = width - 2 * BUBBLE_INNER_MARGIN;
        let paragraph = Paragraph::compile(BUBBLE_FONT_SIZE,
                                           BUBBLE_LINE_HEIGHT,
                                           wrap_width as f32,
                                           prefs,
                                           format);
        let height = 2 * BUBBLE_INNER_MARGIN +
            (paragraph.height().ceil() as i32);
        let view = YouSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            paragraph,
        };
        Box::new(view)
    }
}

impl BubbleView for YouSpeechBubbleView {
    fn rect(&self) -> Rect<i32> { self.rect }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let color = Color3::new(0.5, 0.1, 0.1);
        resources
            .shaders()
            .solid()
            .fill_rect(&matrix, color, self.rect.as_f32());
        let right = (self.rect.right() - BUBBLE_INNER_MARGIN) as f32;
        let left = (right - self.paragraph.width()).floor();
        let top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));
    }
}

//===========================================================================//
