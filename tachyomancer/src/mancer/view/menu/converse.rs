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

use super::super::button::{HoverPulse, Scrollbar, TextButton};
use super::super::paragraph::Paragraph;
use super::list::{list_height_for_num_items, ListIcon, ListView};
use crate::mancer::font::Align;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::{Event, Resources, Sound, Ui};
use crate::mancer::save::{Prefs, Profile};
use crate::mancer::state::{
    ConversationBubble, ConversationExt, Cutscene, GameState, Portrait,
};
use cgmath::{vec2, Matrix4, Point2};
use num_integer::div_mod_floor;
use tachy::geom::{AsFloat, Color3, Color4, MatrixExt, Rect};
use tachy::save::{Chapter, Conversation, Puzzle};

//===========================================================================//

const CHAPTER_LIST_WIDTH: i32 = 120;
const CHAPTER_LIST_HEIGHT: i32 = list_height_for_num_items(5);
const CONV_LIST_WIDTH: i32 = 220;
const LIST_MARGIN_HORZ: i32 = 22;

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

const PUZZLE_HEIGHT: i32 = 50;
const PUZZLE_SPACING: i32 = 2;

const SCROLLBAR_WIDTH: i32 = 18;
const SCROLLBAR_MARGIN: i32 = 8;

//===========================================================================//

pub enum ConverseAction {
    GoToPuzzle(Puzzle),
    PlayCutscene(Cutscene),
}

//===========================================================================//

pub struct ConverseView {
    chapter_list: ListView<Chapter>,
    conv_list: ListView<Conversation>,
    bubbles_list: BubblesListView,
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
        let bubbles_list_left =
            conv_list_left + CONV_LIST_WIDTH + LIST_MARGIN_HORZ;
        let bubbles_list_width = rect.right() - bubbles_list_left;

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
            bubbles_list: BubblesListView::new(
                Rect::new(
                    bubbles_list_left,
                    rect.y,
                    bubbles_list_width,
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
        self.bubbles_list.draw(resources, matrix);
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
                self.bubbles_list.reset(ui, state);
            }
            _ => {}
        }

        // Conversations list:
        if let Some(conv) =
            self.conv_list.on_event(event, ui, &state.current_conversation())
        {
            state.set_current_conversation(conv);
            ui.request_redraw();
            self.bubbles_list.update_conversation(ui, state);
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
            self.reset_for_current_conversation(ui, state);
            return None;
        }

        // Bubbles list:
        match self.bubbles_list.on_event(event, ui) {
            Some(BubblesAction::Complete) => {
                state.mark_current_conversation_complete();
                self.reset_for_current_conversation(ui, state);
            }
            Some(BubblesAction::GoToPuzzle(puzzle)) => {
                return Some(ConverseAction::GoToPuzzle(puzzle));
            }
            Some(BubblesAction::Increment) => {
                state.increment_current_conversation_progress();
                self.bubbles_list.update_conversation(ui, state);
            }
            Some(BubblesAction::MakeChoice(key, value)) => {
                state.set_current_conversation_choice(key, value);
                state.increment_current_conversation_progress();
                self.bubbles_list.update_conversation(ui, state);
            }
            Some(BubblesAction::PlayCutscene(cutscene)) => {
                return Some(ConverseAction::PlayCutscene(cutscene));
            }
            None => {}
        }
        return None;
    }

    pub fn reset_for_current_conversation(
        &mut self,
        ui: &mut Ui,
        state: &GameState,
    ) {
        self.conv_list.set_items(
            ui,
            conv_list_items(state),
            &state.current_conversation(),
        );
        self.bubbles_list.update_conversation(ui, state);
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

#[derive(Clone)]
enum BubblesAction {
    Complete,
    GoToPuzzle(Puzzle),
    Increment,
    MakeChoice(String, String),
    PlayCutscene(Cutscene),
}

//===========================================================================//

struct BubblesListView {
    rect: Rect<i32>,
    conv: Conversation,
    bubbles: Vec<Box<dyn BubbleView>>,
    num_bubbles_shown: usize,
    more_button: Option<MoreButton>,
    scrollbar: Scrollbar,
}

impl BubblesListView {
    fn new(
        rect: Rect<i32>,
        ui: &mut Ui,
        state: &GameState,
    ) -> BubblesListView {
        let scrollbar_rect = Rect::new(
            rect.right() - SCROLLBAR_WIDTH,
            rect.y,
            SCROLLBAR_WIDTH,
            rect.height,
        );
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

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubblesAction> {
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
                    return Some(BubblesAction::Increment);
                }
                if let Some(ref bubble) = self.bubbles.last() {
                    if bubble.is_choice_or_puzzle() {
                        return Some(BubblesAction::Increment);
                    }
                }
                return Some(BubblesAction::Complete);
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
        self.scrollbar.scroll_to(total_height, ui);
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
            let mut bubble_top = if let Some(last) = bubble_views.last() {
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
                    for puzzle in puzzles {
                        bubble_views.push(PuzzleBubbleView::new(
                            bubble_width,
                            bubble_top,
                            puzzle,
                        ));
                        bubble_top += PUZZLE_HEIGHT + PUZZLE_SPACING;
                    }
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
            BUBBLE_FONT_SIZE,
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

trait BubbleView {
    fn rect(&self) -> Rect<i32>;

    fn is_choice_or_puzzle(&self) -> bool {
        false
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>);

    fn on_event(
        &mut self,
        _event: &Event,
        _ui: &mut Ui,
    ) -> Option<BubblesAction> {
        None
    }
}

//===========================================================================//

struct CutsceneBubbleView {
    rect: Rect<i32>,
    button: TextButton<BubblesAction>,
}

impl CutsceneBubbleView {
    fn new(width: i32, top: i32, cutscene: Cutscene) -> Box<dyn BubbleView> {
        let rect = Rect::new(0, top, width, CUTSCENE_BUBBLE_HEIGHT);
        let view = CutsceneBubbleView {
            rect,
            button: TextButton::new(
                rect,
                "Replay cutscene",
                BubblesAction::PlayCutscene(cutscene),
            ),
        };
        Box::new(view)
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
    ) -> Option<BubblesAction> {
        self.button.on_event(event, ui, true)
    }
}

//===========================================================================//

struct NpcSpeechBubbleView {
    rect: Rect<i32>,
    portrait: Portrait,
    paragraph: Paragraph,
}

impl NpcSpeechBubbleView {
    fn new(
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
        let view = NpcSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            portrait,
            paragraph,
        };
        Box::new(view)
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
}

//===========================================================================//

struct PuzzleBubbleView {
    rect: Rect<i32>,
    puzzle: Puzzle,
    hover_pulse: HoverPulse,
}

impl PuzzleBubbleView {
    fn new(width: i32, top: i32, puzzle: Puzzle) -> Box<dyn BubbleView> {
        let view = PuzzleBubbleView {
            rect: Rect::new(0, top, width, PUZZLE_HEIGHT),
            puzzle,
            hover_pulse: HoverPulse::new(),
        };
        Box::new(view)
    }
}

impl BubbleView for PuzzleBubbleView {
    fn rect(&self) -> Rect<i32> {
        self.rect
    }

    fn is_choice_or_puzzle(&self) -> bool {
        true
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        let bg_color = Color4::CYAN0_TRANSLUCENT
            .mix(Color4::CYAN3_TRANSLUCENT, self.hover_pulse.brightness());
        resources.shaders().ui().draw_box4(
            &matrix,
            &rect,
            &Color4::ORANGE5,
            &Color4::CYAN3,
            &bg_color,
        );
        let label = format!("Go to task \"{}\"", self.puzzle.title());
        resources.fonts().bold().draw(
            &matrix,
            BUBBLE_FONT_SIZE,
            Align::MidCenter,
            (rect.x + 0.5 * rect.width, rect.y + 0.5 * rect.height),
            &label,
        );
    }

    fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<BubblesAction> {
        match event {
            Event::ClockTick(tick) => {
                self.hover_pulse.on_clock_tick(tick, ui);
            }
            Event::MouseDown(mouse) => {
                if self.rect.contains_point(mouse.pt) {
                    self.hover_pulse.on_click(ui);
                    return Some(BubblesAction::GoToPuzzle(self.puzzle));
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

struct YouChoiceBubbleView {
    rect: Rect<i32>,
    key: String,
    choices: Vec<(String, String)>,
    hovering: Option<usize>,
}

impl YouChoiceBubbleView {
    fn new(
        width: i32,
        top: i32,
        key: String,
        choices: Vec<(String, String)>,
    ) -> Box<dyn BubbleView> {
        debug_assert!(!choices.is_empty());
        let height = (choices.len() as i32) * (CHOICE_HEIGHT + CHOICE_SPACING)
            - CHOICE_SPACING;
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
    ) -> Option<BubblesAction> {
        match event {
            Event::MouseDown(mouse) => {
                if let Some(index) = self.choice_for_pt(mouse.pt) {
                    let key = self.key.clone();
                    let value = self.choices[index].0.clone();
                    return Some(BubblesAction::MakeChoice(key, value));
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
    fn new(
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
        let view = YouSpeechBubbleView {
            rect: Rect::new(0, top, width, height),
            paragraph,
        };
        Box::new(view)
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
        let right = (self.rect.right() - BUBBLE_INNER_MARGIN) as f32;
        let left = (right - self.paragraph.width()).floor();
        let top = (self.rect.y + BUBBLE_INNER_MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));
    }
}

//===========================================================================//
