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

use super::converse::{ConverseAction, ConverseView};
use super::dialog::{ButtonDialogBox, TextDialogBox};
use super::prefs::{PrefsAction, PrefsView};
use super::puzzle::{PuzzlesAction, PuzzlesView};
use cgmath::{self, Matrix4};
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Keycode, Resources, Sound, Window,
                 WindowOptions};
use tachy::save::{CIRCUIT_NAME_MAX_WIDTH, MenuSection, Puzzle};
use tachy::state::GameState;
use textwrap;

//===========================================================================//

const SECTION_BUTTON_HEIGHT: i32 = 50;
const SECTION_BUTTON_MARGIN_TOP: i32 = 40;
const SECTION_BUTTON_MARGIN_HORZ: i32 = 40;
const SECTION_BUTTON_SPACING: i32 = 20;

const SECTION_MARGIN_TOP: i32 = 30;
const SECTION_MARGIN_BOTTOM: i32 = 40;
const SECTION_MARGIN_HORZ: i32 = SECTION_BUTTON_MARGIN_HORZ;
const SECTION_TOP: i32 = SECTION_BUTTON_MARGIN_TOP + SECTION_BUTTON_HEIGHT +
    SECTION_MARGIN_TOP;

//===========================================================================//

#[derive(Clone)]
pub enum MenuAction {
    GoToPuzzle(Puzzle),
    CopyCircuit,
    DeleteCircuit,
    EditCircuit,
    NewCircuit,
    RenameCircuit(String),
    RebootWindow(WindowOptions),
    NewProfile,
    SwitchProfile(String),
    DeleteProfile,
    QuitGame,
}

//===========================================================================//

pub struct MenuView {
    width: f32,
    height: f32,
    section_buttons: Vec<SectionButton>,
    converse_view: ConverseView,
    prefs_view: PrefsView,
    puzzles_view: PuzzlesView,
    confirmation_dialog: Option<ButtonDialogBox<Option<MenuAction>>>,
    rename_dialog: Option<TextDialogBox>,
}

impl MenuView {
    pub fn new(window: &Window, state: &GameState) -> MenuView {
        let window_size = window.size();
        let section_buttons = vec![
            SectionButton::new(window_size,
                               0,
                               "Navigation",
                               MenuSection::Navigation),
            SectionButton::new(window_size,
                               1,
                               "Messages",
                               MenuSection::Messages),
            SectionButton::new(window_size,
                               2,
                               "Tasks",
                               MenuSection::Puzzles),
            SectionButton::new(window_size,
                               3,
                               "Settings",
                               MenuSection::Prefs),
        ];
        let section_rect =
            Rect::new(SECTION_MARGIN_HORZ,
                      SECTION_TOP,
                      (window_size.width as i32) - 2 * SECTION_MARGIN_HORZ,
                      (window_size.height as i32) - SECTION_TOP -
                          SECTION_MARGIN_BOTTOM);
        MenuView {
            width: window_size.width as f32,
            height: window_size.height as f32,
            section_buttons,
            converse_view: ConverseView::new(section_rect, state),
            prefs_view: PrefsView::new(section_rect, window, state),
            puzzles_view: PuzzlesView::new(section_rect, state),
            confirmation_dialog: None,
            rename_dialog: None,
        }
    }

    pub fn draw(&self, resources: &Resources, state: &GameState) {
        let projection =
            cgmath::ortho(0.0, self.width, self.height, 0.0, -1.0, 1.0);
        let rect = Rect::new(0.0, 0.0, self.width, self.height);
        resources
            .shaders()
            .solid()
            .fill_rect(&projection, (0.2, 0.1, 0.2), rect);
        for button in self.section_buttons.iter() {
            button.draw(resources, &projection, state.menu_section());
        }
        match state.menu_section() {
            MenuSection::Navigation => {
                // TODO
            }
            MenuSection::Messages => {
                self.converse_view.draw(resources, &projection, state);
            }
            MenuSection::Puzzles => {
                self.puzzles_view.draw(resources, &projection, state);
            }
            MenuSection::Prefs => {
                self.prefs_view.draw(resources, &projection, state);
            }
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

    pub fn on_event(&mut self, event: &Event, state: &mut GameState,
                    audio: &mut AudioQueue)
                    -> Option<MenuAction> {
        if let Some(mut dialog) = self.confirmation_dialog.take() {
            match dialog.on_event(event) {
                Some(Some(action)) => return Some(action),
                Some(None) => {}
                None => self.confirmation_dialog = Some(dialog),
            }
            if !event.is_clock_tick() {
                return None;
            }
        }

        if let Some(mut dialog) = self.rename_dialog.take() {
            match dialog.on_event(event, |name| {
                state.is_valid_circuit_rename(name)
            }) {
                Some(Some(name)) => {
                    return Some(MenuAction::RenameCircuit(name));
                }
                Some(None) => {}
                None => self.rename_dialog = Some(dialog),
            }
            if !event.is_clock_tick() {
                return None;
            }
        }

        if let Some(action) = self.on_section_event(event, state, audio) {
            return Some(action);
        }

        let mut next_section: Option<MenuSection> = None;
        for button in self.section_buttons.iter_mut() {
            if let Some(section) =
                button.on_event(event, state.menu_section(), audio)
            {
                next_section = Some(section);
                break;
            }
        }
        if let Some(section) = next_section {
            self.on_section_event(&Event::Unfocus, state, audio);
            state.set_menu_section(section);
        }

        return None;
    }

    fn on_section_event(&mut self, event: &Event, state: &mut GameState,
                        audio: &mut AudioQueue)
                        -> Option<MenuAction> {
        match state.menu_section() {
            MenuSection::Navigation => {
                // TODO
            }
            MenuSection::Messages => {
                match self.converse_view.on_event(event, state) {
                    Some(ConverseAction::Complete) => {
                        state.mark_current_conversation_complete();
                        self.converse_view.update_conversation_list(state);
                        self.converse_view.update_conversation_bubbles(state);
                    }
                    Some(ConverseAction::GoToPuzzle(puzzle)) => {
                        return Some(MenuAction::GoToPuzzle(puzzle));
                    }
                    Some(ConverseAction::Increment) => {
                        state.increment_current_conversation_progress();
                        self.converse_view.update_conversation_bubbles(state);
                    }
                    Some(ConverseAction::MakeChoice(key, value)) => {
                        state.set_current_conversation_choice(key, value);
                        state.increment_current_conversation_progress();
                        self.converse_view.update_conversation_bubbles(state);
                    }
                    None => {}
                }
            }
            MenuSection::Puzzles => {
                match self.puzzles_view.on_event(event, state) {
                    Some(PuzzlesAction::Copy) => {
                        return Some(MenuAction::CopyCircuit);
                    }
                    Some(PuzzlesAction::Delete) => {
                        self.unfocus(state);
                        let size = RectSize::new(self.width as i32,
                                                 self.height as i32);
                        let text = format!("Really delete {}?",
                                           state.circuit_name());
                        let cancel_button =
                            ("Cancel", None, Some(Keycode::Escape));
                        let delete_button =
                            ("Delete", Some(MenuAction::DeleteCircuit), None);
                        let buttons = &[cancel_button, delete_button];
                        self.confirmation_dialog =
                            Some(ButtonDialogBox::new(size, &text, buttons));
                        return None;
                    }
                    Some(PuzzlesAction::Edit) => {
                        return Some(MenuAction::EditCircuit);
                    }
                    Some(PuzzlesAction::New) => {
                        return Some(MenuAction::NewCircuit);
                    }
                    Some(PuzzlesAction::Rename) => {
                        self.unfocus(state);
                        let size = RectSize::new(self.width as i32,
                                                 self.height as i32);
                        let text = "Choose new circuit name:";
                        let initial = state.circuit_name();
                        let dialog =
                            TextDialogBox::new(size,
                                               text,
                                               initial,
                                               CIRCUIT_NAME_MAX_WIDTH);
                        self.rename_dialog = Some(dialog);
                        return None;
                    }
                    None => {}
                }
            }
            MenuSection::Prefs => {
                match self.prefs_view.on_event(event, state, audio) {
                    Some(PrefsAction::RebootWindow(options)) => {
                        return Some(MenuAction::RebootWindow(options));
                    }
                    Some(PrefsAction::NewProfile) => {
                        return Some(MenuAction::NewProfile);
                    }
                    Some(PrefsAction::SwitchProfile(name)) => {
                        return Some(MenuAction::SwitchProfile(name));
                    }
                    Some(PrefsAction::DeleteProfile) => {
                        self.unfocus(state);
                        let size = RectSize::new(self.width as i32,
                                                 self.height as i32);
                        let text = format!("Are you sure you want \
                                            to delete all progress\n\
                                            on profile \"{}\"?\n\n\
                                            This cannot be undone!",
                                           state.profile().unwrap().name());
                        let cancel_button =
                            ("Cancel", None, Some(Keycode::Escape));
                        let delete_button =
                            ("Delete", Some(MenuAction::DeleteProfile), None);
                        let buttons = &[cancel_button, delete_button];
                        self.confirmation_dialog =
                            Some(ButtonDialogBox::new(size, &text, buttons));
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

    pub fn show_error(&mut self, state: &mut GameState, unable: &str,
                      error: &str) {
        debug_log!("ERROR: Unable to {}: {}", unable, error);
        self.unfocus(state);
        let size = RectSize::new(self.width as i32, self.height as i32);
        let text = format!("ERROR: Unable to {}.\n\n{}", unable, error);
        let text = textwrap::fill(&text, 64);
        let buttons = &[("OK", None, Some(Keycode::Return))];
        let dialog = ButtonDialogBox::new(size, &text, buttons);
        self.confirmation_dialog = Some(dialog);
    }

    pub fn go_to_current_puzzle(&mut self, state: &mut GameState) {
        self.unfocus(state);
        state.set_menu_section(MenuSection::Puzzles);
        self.update_circuit_list(state);
    }

    pub fn update_circuit_list(&mut self, state: &GameState) {
        self.puzzles_view.update_circuit_list(state);
    }

    pub fn update_conversation(&mut self, state: &GameState) {
        self.converse_view.update_conversation_list(state);
        self.converse_view.update_conversation_bubbles(state);
    }

    pub fn update_profile_list(&mut self, state: &GameState) {
        self.prefs_view.update_profile_list(state);
    }

    pub fn update_puzzle_list(&mut self, state: &GameState) {
        self.puzzles_view.update_puzzle_list(state);
    }

    fn unfocus(&mut self, state: &mut GameState) {
        let mut audio = AudioQueue::new();
        self.converse_view.on_event(&Event::Unfocus, state);
        self.prefs_view.on_event(&Event::Unfocus, state, &mut audio);
        self.puzzles_view.on_event(&Event::Unfocus, state);
    }
}

//===========================================================================//

struct SectionButton {
    rect: Rect<i32>,
    label: &'static str,
    section: MenuSection,
}

impl SectionButton {
    fn new(window_size: RectSize<i32>, index: i32, label: &'static str,
           section: MenuSection)
           -> SectionButton {
        let width = (window_size.width - 2 * SECTION_BUTTON_MARGIN_HORZ -
                         3 * SECTION_BUTTON_SPACING) / 4;
        let left = SECTION_BUTTON_MARGIN_HORZ +
            index * (width + SECTION_BUTTON_SPACING);
        let rect = Rect::new(left,
                             SECTION_BUTTON_MARGIN_TOP,
                             width,
                             SECTION_BUTTON_HEIGHT);
        SectionButton {
            rect,
            label,
            section,
        }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
            current_section: MenuSection) {
        let color = if self.section == current_section {
            (0.75, 0.0, 0.0)
        } else {
            (0.0, 0.25, 0.0)
        };
        let rect = self.rect.as_f32();
        resources.shaders().solid().fill_rect(matrix, color, rect);
        resources.fonts().roman().draw(&matrix,
                                       20.0,
                                       Align::MidCenter,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height),
                                       self.label);
    }

    fn on_event(&mut self, event: &Event, current_section: MenuSection,
                audio: &mut AudioQueue)
                -> Option<MenuSection> {
        match event {
            Event::MouseDown(mouse) => {
                if self.section != current_section &&
                    self.rect.contains_point(mouse.pt)
                {
                    audio.play_sound(Sound::Beep);
                    return Some(self.section);
                }
            }
            _ => {}
        }
        return None;
    }
}

//===========================================================================//
