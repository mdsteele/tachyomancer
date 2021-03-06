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

mod converse;
mod credits;
mod list;
mod nav;
mod prefs;
mod puzzle;

use self::converse::{ConverseAction, ConverseView};
use self::nav::NavigationView;
use self::prefs::{PrefsAction, PrefsView};
use self::puzzle::{PuzzlesAction, PuzzlesView};
use super::background::{background_for_chapter, BackgroundView};
use super::button::RadioButton;
use super::dialog::{ButtonDialogBox, DialogAction, TextDialogBox};
use super::paragraph::Paragraph;
use crate::mancer::gui::{
    ClockEventData, Cursor, Event, Keycode, Music, Resources, Ui, Window,
    WindowOptions,
};
use crate::mancer::save::{MenuSection, CIRCUIT_NAME_MAX_CHARS};
use crate::mancer::state::{Cutscene, GameState};
use cgmath::{self, Matrix4};
use tachy::geom::{AsFloat, MatrixExt, Rect, RectSize};
use tachy::save::{Chapter, Conversation, Puzzle};

//===========================================================================//

const SECTION_BUTTON_HEIGHT: i32 = 50;
const SECTION_BUTTON_MARGIN_TOP: i32 = 40;
const SECTION_BUTTON_MARGIN_HORZ: i32 = 40;
const SECTION_BUTTON_SPACING: i32 = 20;

const SECTION_MARGIN_TOP: i32 = 30;
const SECTION_MARGIN_BOTTOM: i32 = 40;
const SECTION_MARGIN_HORZ: i32 = SECTION_BUTTON_MARGIN_HORZ;
const SECTION_TOP: i32 =
    SECTION_BUTTON_MARGIN_TOP + SECTION_BUTTON_HEIGHT + SECTION_MARGIN_TOP;

//===========================================================================//

#[derive(Clone)]
pub enum MenuAction {
    GoToPuzzle(Puzzle),
    PlayCutscene(Cutscene),
    UnlockPuzzles(Vec<Puzzle>),
    CopyCircuit,
    DeleteCircuit,
    EditCircuit,
    RenameCircuit(String),
    RebootWindow(WindowOptions),
    NewProfile,
    SwitchProfile(String),
    DeleteProfile(String),
    QuitGame,
}

//===========================================================================//

pub struct MenuView {
    size: RectSize<i32>,
    background: Box<dyn BackgroundView>,

    section_buttons: Vec<RadioButton<MenuSection>>,
    navigation_view: NavigationView,
    converse_view: ConverseView,
    prefs_view: PrefsView,
    puzzles_view: PuzzlesView,

    confirmation_dialog: Option<ButtonDialogBox<Option<MenuAction>>>,
    rename_dialog: Option<TextDialogBox>,

    left_section: MenuSection,
    right_section: MenuSection,
    section_anim: f32,
}

impl MenuView {
    pub fn new(window: &mut Window, state: &GameState) -> MenuView {
        let size = window.size();
        let section_buttons = vec![
            section_button(size, 0, "Navigation", MenuSection::Navigation),
            section_button(size, 1, "Messages", MenuSection::Messages),
            section_button(size, 2, "Tasks", MenuSection::Puzzles),
            section_button(size, 3, "Settings", MenuSection::Prefs),
        ];
        let section_rect = Rect::new(
            SECTION_MARGIN_HORZ,
            SECTION_TOP,
            size.width - 2 * SECTION_MARGIN_HORZ,
            size.height - SECTION_TOP - SECTION_MARGIN_BOTTOM,
        );

        let latest_chapter = state.latest_chapter();

        let prefs_view = PrefsView::new(section_rect, window, state);
        let mut ui = window.ui();
        let navigation_view =
            NavigationView::new(size.as_f32(), state, latest_chapter);
        let converse_view = ConverseView::new(section_rect, &mut ui, state);
        let puzzles_view = PuzzlesView::new(section_rect, &mut ui, state);

        ui.audio().play_music(music_for_chapter(latest_chapter));

        MenuView {
            size,
            background: background_for_chapter(latest_chapter, size.as_f32()),
            section_buttons,
            navigation_view,
            converse_view,
            prefs_view,
            puzzles_view,
            confirmation_dialog: None,
            rename_dialog: None,
            left_section: state.menu_section(),
            right_section: state.menu_section(),
            section_anim: 0.0,
        }
    }

    pub fn draw(&self, resources: &Resources, state: &GameState) {
        self.background.draw(resources);
        let size = self.size.as_f32();
        let projection =
            cgmath::ortho(0.0, size.width, size.height, 0.0, -10.0, 10.0);
        if self.left_section == self.right_section {
            self.draw_section(
                resources,
                &projection,
                self.left_section,
                state,
            );
        } else {
            let matrix1 = projection
                * Matrix4::trans2(-size.width * self.section_anim, 0.0);
            self.draw_section(resources, &matrix1, self.left_section, state);
            let matrix2 = projection
                * Matrix4::trans2(size.width * (1.0 - self.section_anim), 0.0);
            self.draw_section(resources, &matrix2, self.right_section, state);
        }
        for button in self.section_buttons.iter() {
            let enabled = button.value() != &MenuSection::Puzzles
                || state.are_any_puzzles_unlocked();
            button.draw(
                resources,
                &projection,
                &state.menu_section(),
                enabled,
            );
        }
        if let Some(ref dialog) = self.rename_dialog {
            dialog.draw(resources, &projection, |name| {
                state.is_valid_circuit_rename(name)
            });
        }
        if let Some(ref dialog) = self.confirmation_dialog {
            dialog.draw(resources, &projection);
        }
    }

    fn draw_section(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        section: MenuSection,
        state: &GameState,
    ) {
        match section {
            MenuSection::Navigation => {
                self.navigation_view.draw(resources, matrix);
            }
            MenuSection::Messages => {
                self.converse_view.draw(resources, matrix, state);
            }
            MenuSection::Puzzles => {
                self.puzzles_view.draw(resources, matrix, state);
            }
            MenuSection::Prefs => {
                self.prefs_view.draw(resources, matrix, state);
            }
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<MenuAction> {
        ui.cursor().request(Cursor::default());
        self.background.on_event(event, ui);
        match event {
            Event::ClockTick(tick) => {
                debug_assert!(self.left_section <= self.right_section);
                let goal_section = state.menu_section();
                if self.left_section == self.right_section {
                    if goal_section < self.left_section {
                        self.left_section = goal_section;
                        self.section_anim = 1.0;
                    } else if goal_section > self.left_section {
                        self.right_section = goal_section;
                        self.section_anim = 0.0;
                    }
                }
                if self.left_section != self.right_section {
                    let mut anim_goal = if goal_section == self.left_section {
                        0.0
                    } else if goal_section == self.right_section {
                        1.0
                    } else if goal_section < self.left_section {
                        -1.0
                    } else if goal_section > self.right_section {
                        2.0
                    } else if self.section_anim < 0.5 {
                        -1.0
                    } else {
                        2.0
                    };
                    self.section_anim =
                        track_towards(self.section_anim, anim_goal, tick);
                    ui.request_redraw();
                    if self.section_anim < 0.0 {
                        debug_assert!(self.section_anim >= -1.0);
                        if goal_section < self.left_section {
                            self.right_section = self.left_section;
                            self.left_section = goal_section;
                            self.section_anim += 1.0;
                            anim_goal = 0.0;
                        } else {
                            debug_assert!(goal_section < self.right_section);
                            self.right_section = goal_section;
                            self.section_anim = -self.section_anim;
                            anim_goal = 1.0;
                        }
                    } else if self.section_anim > 1.0 {
                        debug_assert!(self.section_anim <= 2.0);
                        if goal_section > self.right_section {
                            self.left_section = self.right_section;
                            self.right_section = goal_section;
                            self.section_anim -= 1.0;
                            anim_goal = 1.0;
                        } else {
                            debug_assert!(goal_section > self.left_section);
                            self.left_section = goal_section;
                            self.section_anim = 2.0 - self.section_anim;
                            anim_goal = 0.0;
                        }
                    }
                    debug_assert!(
                        self.section_anim >= 0.0 && self.section_anim <= 1.0
                    );
                    if self.section_anim == anim_goal {
                        self.left_section = goal_section;
                        self.right_section = goal_section;
                    }
                }
            }
            Event::Debug(key, value) if key == "SetBackground" => {
                if let Ok(chapter) = value.parse::<Chapter>() {
                    self.background =
                        background_for_chapter(chapter, self.size.as_f32());
                    self.navigation_view.refresh_indicators(state, chapter);
                    ui.request_redraw();
                }
            }
            Event::Debug(key, value) if key == "ResetScores" => {
                if value.is_empty() {
                    state.reset_local_scores(state.current_puzzle());
                } else if let Ok(puzzle) = value.parse::<Puzzle>() {
                    state.reset_local_scores(puzzle);
                }
                self.puzzles_view.clear_score_graph_cache();
                ui.request_redraw();
            }
            Event::Debug(key, value) if key == "UnlockPuzzle" => {
                if let Ok(puzzle) = value.parse::<Puzzle>() {
                    return Some(MenuAction::GoToPuzzle(puzzle));
                }
            }
            _ => {}
        }

        if let Some(mut dialog) = self.confirmation_dialog.take() {
            match dialog.on_event(event, ui) {
                Some(Some(action)) => return Some(action),
                Some(None) => {}
                None => self.confirmation_dialog = Some(dialog),
            }
            if !event.is_clock_tick() {
                return None;
            }
        }

        if let Some(mut dialog) = self.rename_dialog.take() {
            match dialog.on_event(event, ui, |name| {
                state.is_valid_circuit_rename(name)
            }) {
                Some(DialogAction::Value(name)) => {
                    return Some(MenuAction::RenameCircuit(name));
                }
                Some(DialogAction::Cancel) => {}
                None => self.rename_dialog = Some(dialog),
            }
            if !event.is_clock_tick() {
                return None;
            }
        }

        if self.left_section == self.right_section || event.is_clock_tick() {
            if let Some(action) = self.on_section_event(event, ui, state) {
                return Some(action);
            }
        }

        let mut next_section: Option<MenuSection> = None;
        for button in self.section_buttons.iter_mut() {
            let enabled = button.value() != &MenuSection::Puzzles
                || state.are_any_puzzles_unlocked();
            if let Some(section) =
                button.on_event(event, ui, &state.menu_section(), enabled)
            {
                next_section = Some(section);
                break;
            }
        }
        if let Some(section) = next_section {
            self.on_section_event(&Event::Unfocus, ui, state);
            state.set_menu_section(section);
        }

        return None;
    }

    fn on_section_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<MenuAction> {
        match state.menu_section() {
            MenuSection::Navigation => {
                self.navigation_view.on_event(event, ui);
            }
            MenuSection::Messages => {
                match self.converse_view.on_event(event, ui, state) {
                    Some(ConverseAction::GoToPuzzle(puzzle)) => {
                        return Some(MenuAction::GoToPuzzle(puzzle));
                    }
                    Some(ConverseAction::PlayCutscene(cutscene)) => {
                        return Some(MenuAction::PlayCutscene(cutscene));
                    }
                    Some(ConverseAction::UnlockPuzzles(puzzles)) => {
                        return Some(MenuAction::UnlockPuzzles(puzzles));
                    }
                    None => {}
                }
            }
            MenuSection::Puzzles => {
                match self.puzzles_view.on_event(event, ui, state) {
                    Some(PuzzlesAction::GoToConversation(conv)) => {
                        self.go_back_to_conversation(conv, ui, state);
                        return None;
                    }
                    Some(PuzzlesAction::Copy) => {
                        return Some(MenuAction::CopyCircuit);
                    }
                    Some(PuzzlesAction::Delete) => {
                        self.unfocus(ui, state);
                        let format = format!(
                            "Really delete {}?",
                            Paragraph::escape(state.circuit_name())
                        );
                        let cancel_button =
                            ("Cancel", None, Some(Keycode::Escape));
                        let delete_button =
                            ("Delete", Some(MenuAction::DeleteCircuit), None);
                        let buttons = &[cancel_button, delete_button];
                        self.confirmation_dialog = Some(ButtonDialogBox::new(
                            self.size,
                            state.prefs(),
                            &format,
                            buttons,
                        ));
                        return None;
                    }
                    Some(PuzzlesAction::Edit) => {
                        return Some(MenuAction::EditCircuit);
                    }
                    Some(PuzzlesAction::Rename) => {
                        self.unfocus(ui, state);
                        let dialog = TextDialogBox::new(
                            self.size,
                            state.prefs(),
                            "Choose new circuit name:",
                            state.circuit_name(),
                            CIRCUIT_NAME_MAX_CHARS,
                        );
                        self.rename_dialog = Some(dialog);
                        return None;
                    }
                    None => {}
                }
            }
            MenuSection::Prefs => {
                match self.prefs_view.on_event(event, ui, state) {
                    Some(PrefsAction::RebootWindow(options)) => {
                        return Some(MenuAction::RebootWindow(options));
                    }
                    Some(PrefsAction::NewProfile) => {
                        return Some(MenuAction::NewProfile);
                    }
                    Some(PrefsAction::SwitchProfile(name)) => {
                        return Some(MenuAction::SwitchProfile(name));
                    }
                    Some(PrefsAction::DeleteProfile(name)) => {
                        self.unfocus(ui, state);
                        let format = format!(
                            "Are you sure you want \
                             to delete all progress\n\
                             on profile \"{}\"?\n\n\
                             This cannot be undone!",
                            Paragraph::escape(&name)
                        );
                        let cancel_button =
                            ("Cancel", None, Some(Keycode::Escape));
                        let delete_button = (
                            "Delete",
                            Some(MenuAction::DeleteProfile(name)),
                            None,
                        );
                        let buttons = &[cancel_button, delete_button];
                        self.confirmation_dialog = Some(ButtonDialogBox::new(
                            self.size,
                            state.prefs(),
                            &format,
                            buttons,
                        ));
                        return None;
                    }
                    Some(PrefsAction::QuitGame) => {
                        return Some(MenuAction::QuitGame);
                    }
                    None => {}
                }
            }
        }
        return None;
    }

    pub fn show_error(
        &mut self,
        ui: &mut Ui,
        state: &mut GameState,
        unable: &str,
        error: &str,
    ) {
        debug_log!("ERROR: Unable to {}: {}", unable, error);
        // TODO: Play sound for error dialog popup.
        self.unfocus(ui, state);
        let format = format!(
            "$R$*ERROR:$*$D Unable to {}.\n\n{}",
            unable,
            Paragraph::escape(error)
        );
        let buttons = &[("OK", None, Some(Keycode::Return))];
        let dialog =
            ButtonDialogBox::new(self.size, state.prefs(), &format, buttons);
        self.confirmation_dialog = Some(dialog);
    }

    fn go_back_to_conversation(
        &mut self,
        conv: Conversation,
        ui: &mut Ui,
        state: &mut GameState,
    ) {
        state.set_current_conversation(conv);
        self.unfocus(ui, state);
        state.set_menu_section(MenuSection::Messages);
        self.converse_view.jump_to_current_conversation_from(
            state.current_puzzle(),
            ui,
            state,
        );
    }

    pub fn go_to_current_puzzle(
        &mut self,
        ui: &mut Ui,
        state: &mut GameState,
    ) {
        self.unfocus(ui, state);
        state.set_menu_section(MenuSection::Puzzles);
        self.update_circuit_list(ui, state);
    }

    pub fn update_circuit_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.puzzles_view.update_circuit_list(ui, state);
    }

    pub fn update_puzzle_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.puzzles_view.update_puzzle_list(ui, state);
    }

    fn unfocus(&mut self, ui: &mut Ui, state: &mut GameState) {
        self.navigation_view.on_event(&Event::Unfocus, ui);
        self.converse_view.on_event(&Event::Unfocus, ui, state);
        self.prefs_view.on_event(&Event::Unfocus, ui, state);
        self.puzzles_view.on_event(&Event::Unfocus, ui, state);
    }
}

//===========================================================================//

fn music_for_chapter(chapter: Chapter) -> Vec<Music> {
    match chapter {
        Chapter::Odyssey => vec![Music::PitchBlack],
        _ => vec![Music::Aduro], // TODO: specify music for other chapters
    }
}

fn section_button(
    window_size: RectSize<i32>,
    index: i32,
    label: &str,
    section: MenuSection,
) -> RadioButton<MenuSection> {
    let width = (window_size.width
        - 2 * SECTION_BUTTON_MARGIN_HORZ
        - 3 * SECTION_BUTTON_SPACING)
        / 4;
    let left =
        SECTION_BUTTON_MARGIN_HORZ + index * (width + SECTION_BUTTON_SPACING);
    let rect = Rect::new(
        left,
        SECTION_BUTTON_MARGIN_TOP,
        width,
        SECTION_BUTTON_HEIGHT,
    );
    RadioButton::new(rect, label, section)
}

fn track_towards(current: f32, goal: f32, tick: &ClockEventData) -> f32 {
    let tracking_base: f64 = 0.0000001; // smaller = faster tracking
    let threshold: f64 = 0.001; // Once we're this close, snap to goal.
    let difference = (goal as f64) - (current as f64);
    if difference.abs() < threshold {
        goal
    } else {
        ((current as f64)
            + difference * (1.0 - tracking_base.powf(tick.elapsed)))
            as f32
    }
}

//===========================================================================//
