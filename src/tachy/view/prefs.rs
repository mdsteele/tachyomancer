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

use super::button::{RadioButton, TextButton};
use super::list::ListView;
use cgmath::Matrix4;
use tachy::geom::Rect;
use tachy::gui::{Event, Resources};
use tachy::state::GameState;

//===========================================================================//

const PANE_BUTTON_SPACING: i32 = 24;
const PANE_BUTTON_WIDTH: i32 = 180;

//===========================================================================//

#[derive(Clone)]
pub enum PrefsAction {
    NewProfile,
    SwitchProfile(String),
    QuitGame,
}

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PrefsPane {
    Hotkeys,
    AudioVideo,
    Profiles,
    Credits,
}

const PANES: &[(PrefsPane, &str)] = &[
    (PrefsPane::Hotkeys, "Controls"),
    (PrefsPane::AudioVideo, "Audio/Video"),
    (PrefsPane::Profiles, "Profiles"),
    (PrefsPane::Credits, "Credits"),
];

//===========================================================================//

pub struct PrefsView {
    current_pane: PrefsPane,
    pane_buttons: Vec<RadioButton<PrefsPane>>,
    quit_button: TextButton<PrefsAction>,
    profiles_list: ListView<String>,
    new_button: TextButton<PrefsAction>,
}

impl PrefsView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PrefsView {
        debug_assert!(state.profile().is_some());

        let num_panes = PANES.len() as i32;
        let pane_button_height = (rect.height + PANE_BUTTON_SPACING) /
            (num_panes + 1) -
            PANE_BUTTON_SPACING;
        let pane_buttons = PANES
            .iter()
            .enumerate()
            .map(|(index, &(pane, label))| {
                let top = rect.y +
                    (index as i32) *
                        (pane_button_height + PANE_BUTTON_SPACING);
                let rect = Rect::new(rect.x,
                                     top,
                                     PANE_BUTTON_WIDTH,
                                     pane_button_height);
                RadioButton::new(rect, label, pane)
            })
            .collect();

        let quit_button_top = rect.y +
            num_panes * (pane_button_height + PANE_BUTTON_SPACING);
        let quit_button_height = rect.height - (quit_button_top - rect.y);
        let quit_button = TextButton::new(Rect::new(rect.x,
                                                    quit_button_top,
                                                    PANE_BUTTON_WIDTH,
                                                    quit_button_height),
                                          "Exit Game",
                                          PrefsAction::QuitGame);

        let current_profile_name = state.profile().unwrap().name();
        let list_items = state
            .savedir()
            .profile_names()
            .map(|name| (name.to_string(), name.to_string()))
            .collect();

        PrefsView {
            current_pane: PrefsPane::Hotkeys,
            pane_buttons,
            quit_button,
            profiles_list: ListView::new(Rect::new(rect.x + PANE_BUTTON_WIDTH +
                                                       PANE_BUTTON_SPACING,
                                                   rect.y,
                                                   300,
                                                   rect.height),
                                         current_profile_name,
                                         list_items),
            new_button: TextButton::new(Rect::new(rect.right() - 150,
                                                  rect.bottom() - 40,
                                                  150,
                                                  40),
                                        "New Profile",
                                        PrefsAction::NewProfile),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        debug_assert!(state.profile().is_some());
        for button in self.pane_buttons.iter() {
            button.draw(resources, matrix, &self.current_pane);
        }
        self.quit_button.draw(resources, matrix, true);

        match self.current_pane {
            PrefsPane::Hotkeys => {
                // TODO
            }
            PrefsPane::AudioVideo => {
                // TODO
            }
            PrefsPane::Profiles => {
                let current_profile_name = state.profile().unwrap().name();
                self.profiles_list
                    .draw(resources, matrix, current_profile_name);
                self.new_button.draw(resources, matrix, true);
            }
            PrefsPane::Credits => {
                // TODO
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PrefsAction> {
        debug_assert!(state.profile().is_some());
        match self.current_pane {
            PrefsPane::Hotkeys => {
                // TODO
            }
            PrefsPane::AudioVideo => {
                // TODO
            }
            PrefsPane::Profiles => {
                let current_profile_name = state.profile().unwrap().name();
                if let Some(profile_name) =
                    self.profiles_list
                        .handle_event(event, current_profile_name)
                {
                    return Some(PrefsAction::SwitchProfile(profile_name));
                }
                if let Some(action) = self.new_button
                    .handle_event(event, true)
                {
                    return Some(action);
                }
            }
            PrefsPane::Credits => {
                // TODO
            }
        }

        let mut next_pane: Option<PrefsPane> = None;
        for button in self.pane_buttons.iter_mut() {
            if let Some(pane) = button
                .handle_event(event, &self.current_pane)
            {
                next_pane = Some(pane);
                break;
            }
        }
        if let Some(pane) = next_pane {
            self.unfocus();
            self.current_pane = pane;
        }
        if let Some(action) = self.quit_button.handle_event(event, true) {
            return Some(action);
        }

        return None;
    }

    pub fn unfocus(&mut self) { self.profiles_list.unfocus(); }
}

//===========================================================================//
