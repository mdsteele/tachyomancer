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

use super::prefs::{PrefsAction, PrefsView};
use super::puzzle::{PuzzlesAction, PuzzlesView};
use cgmath::{self, Matrix4};
use tachy::font::Align;
use tachy::geom::{Rect, RectSize};
use tachy::gui::{AudioQueue, Event, Resources, Sound};
use tachy::save::MenuSection;
use tachy::state::GameState;

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

pub enum MenuAction {
    EditCircuit,
    NewCircuit,
    NewProfile,
    SwitchProfile(String),
}

//===========================================================================//

pub struct MenuView {
    width: f32,
    height: f32,
    section_buttons: Vec<SectionButton>,
    prefs_view: PrefsView,
    puzzles_view: PuzzlesView,
}

impl MenuView {
    pub fn new(window_size: RectSize<u32>, state: &GameState) -> MenuView {
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
            prefs_view: PrefsView::new(section_rect, state),
            puzzles_view: PuzzlesView::new(section_rect, state),
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
            MenuSection::Puzzles => {
                self.puzzles_view.draw(resources, &projection, state);
            }
            MenuSection::Prefs => {
                self.prefs_view.draw(resources, &projection, state);
            }
            _ => {} // TODO
        }
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState,
                        audio: &mut AudioQueue)
                        -> Option<MenuAction> {
        match state.menu_section() {
            MenuSection::Puzzles => {
                match self.puzzles_view.handle_event(event, state) {
                    Some(PuzzlesAction::Edit) => {
                        return Some(MenuAction::EditCircuit);
                    }
                    Some(PuzzlesAction::New) => {
                        return Some(MenuAction::NewCircuit);
                    }
                    None => {}
                }
            }
            MenuSection::Prefs => {
                match self.prefs_view.handle_event(event, state) {
                    Some(PrefsAction::NewProfile) => {
                        return Some(MenuAction::NewProfile);
                    }
                    Some(PrefsAction::SwitchProfile(name)) => {
                        return Some(MenuAction::SwitchProfile(name));
                    }
                    None => {}
                }
            }
            _ => {} // TODO
        }
        for button in self.section_buttons.iter_mut() {
            if let Some(section) =
                button.handle_event(event, state.menu_section(), audio)
            {
                state.set_menu_section(section);
                self.prefs_view.unfocus();
                self.puzzles_view.unfocus();
            }
        }
        return None;
    }
}

//===========================================================================//

struct SectionButton {
    rect: Rect<i32>,
    label: &'static str,
    section: MenuSection,
}

impl SectionButton {
    fn new(window_size: RectSize<u32>, index: i32, label: &'static str,
           section: MenuSection)
           -> SectionButton {
        let width = ((window_size.width as i32) -
                         2 * SECTION_BUTTON_MARGIN_HORZ -
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
                                       Align::Center,
                                       (rect.x + 0.5 * rect.width,
                                        rect.y + 0.5 * rect.height - 10.0),
                                       self.label);
    }

    fn handle_event(&mut self, event: &Event, current_section: MenuSection,
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
