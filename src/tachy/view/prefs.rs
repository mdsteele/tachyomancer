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

use super::button::{Checkbox, HotkeyBox, HotkeyBoxAction, RadioButton,
                    Slider, SliderAction, TextButton};
use super::list::ListView;
use cgmath::{Matrix4, Point2};
use tachy::geom::Rect;
use tachy::gui::{AudioQueue, Event, Resources, Sound};
use tachy::save::Hotkey;
use tachy::state::GameState;

//===========================================================================//

const PANE_BUTTON_SPACING: i32 = 24;
const PANE_BUTTON_WIDTH: i32 = 180;

//===========================================================================//

#[derive(Clone)]
pub enum PrefsAction {
    NewProfile,
    SwitchProfile(String),
    DeleteProfile,
    QuitGame,
}

//===========================================================================//

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PrefsPane {
    AudioVideo,
    Hotkeys,
    Profiles,
    Credits,
}

const PANES: &[(PrefsPane, &str)] = &[
    (PrefsPane::AudioVideo, "Audio/Video"),
    (PrefsPane::Hotkeys, "Controls"),
    (PrefsPane::Profiles, "Profiles"),
    (PrefsPane::Credits, "Credits"),
];

//===========================================================================//

pub struct PrefsView {
    current_pane: PrefsPane,
    pane_buttons: Vec<RadioButton<PrefsPane>>,
    quit_button: TextButton<PrefsAction>,
    audio_video_pane: AudioVideoPane,
    hotkeys_pane: HotkeysPane,
    profiles_pane: ProfilesPane,
}

impl PrefsView {
    pub fn new(rect: Rect<i32>, state: &GameState) -> PrefsView {
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

        let pane_offset = PANE_BUTTON_WIDTH + PANE_BUTTON_SPACING;
        let pane_rect = Rect::new(rect.x + pane_offset,
                                  rect.y,
                                  rect.width - pane_offset,
                                  rect.height);
        let audio_video_pane = AudioVideoPane::new(pane_rect, state);
        let hotkeys_pane = HotkeysPane::new(pane_rect);
        let profiles_pane = ProfilesPane::new(pane_rect, state);

        PrefsView {
            current_pane: PrefsPane::AudioVideo,
            pane_buttons,
            quit_button,
            audio_video_pane,
            hotkeys_pane,
            profiles_pane,
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
            PrefsPane::AudioVideo => {
                self.audio_video_pane.draw(resources, matrix, state);
            }
            PrefsPane::Hotkeys => {
                self.hotkeys_pane.draw(resources, matrix, state);
            }
            PrefsPane::Profiles => {
                self.profiles_pane.draw(resources, matrix, state);
            }
            PrefsPane::Credits => {
                // TODO
            }
        }
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState,
                        audio: &mut AudioQueue)
                        -> Option<PrefsAction> {
        debug_assert!(state.profile().is_some());
        if let Some(action) = self.handle_pane_event(event, state, audio) {
            return Some(action);
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
            self.handle_pane_event(&Event::Unfocus, state, audio);
            self.current_pane = pane;
        }
        if let Some(action) = self.quit_button.handle_event(event, true) {
            return Some(action);
        }

        return None;
    }

    fn handle_pane_event(&mut self, event: &Event, state: &mut GameState,
                         audio: &mut AudioQueue)
                         -> Option<PrefsAction> {
        match self.current_pane {
            PrefsPane::AudioVideo => {
                self.audio_video_pane.handle_event(event, state, audio)
            }
            PrefsPane::Hotkeys => self.hotkeys_pane.handle_event(event, state),
            PrefsPane::Profiles => {
                self.profiles_pane.handle_event(event, state)
            }
            PrefsPane::Credits => {
                None // TODO
            }
        }
    }

    pub fn update_profile_list(&mut self, state: &GameState) {
        self.profiles_pane.update_profile_list(state);
    }
}

//===========================================================================//

pub struct AudioVideoPane {
    antialias_checkbox: Checkbox,
    sound_volume_slider: Slider,
}

impl AudioVideoPane {
    pub fn new(rect: Rect<i32>, state: &GameState) -> AudioVideoPane {
        let antialias_checkbox =
            Checkbox::new(Point2::new(rect.x, rect.y + 20), "Antialiasing");
        let sound_volume_slider =
            Slider::new(Rect::new(rect.x, rect.y + 80, rect.width, 30),
                        state.prefs().sound_volume_percent(),
                        100);
        AudioVideoPane {
            antialias_checkbox,
            sound_volume_slider,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        self.antialias_checkbox
            .draw(resources, matrix, state.prefs().antialiasing(), true);
        self.sound_volume_slider.draw(resources, matrix);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState,
                        audio: &mut AudioQueue)
                        -> Option<PrefsAction> {
        if let Some(checked) =
            self.antialias_checkbox
                .handle_event(event, state.prefs().antialiasing(), true)
        {
            state.prefs_mut().set_antialiasing(checked);
        }
        match self.sound_volume_slider.handle_event(event) {
            Some(SliderAction::Update(volume)) => {
                state.prefs_mut().set_sound_volume_percent(volume);
                audio.set_sound_volume_percent(volume);
            }
            Some(SliderAction::Release) => {
                audio.play_sound(Sound::Beep);
            }
            None => {}
        }
        None
    }
}

//===========================================================================//

pub struct HotkeysPane {
    hotkey_boxes: Vec<HotkeyBox>,
    defaults_button: TextButton<()>,
}

impl HotkeysPane {
    pub fn new(rect: Rect<i32>) -> HotkeysPane {
        let left = rect.x;
        let mut top = rect.y - 16;
        let hotkey_boxes = Hotkey::all()
            .map(|hotkey| {
                     top += 32;
                     HotkeyBox::new(Point2::new(left, top), hotkey)
                 })
            .collect();
        let defaults_button_rect =
            Rect::new(rect.right() - 200, rect.bottom() - 40, 200, 40);
        let defaults_button =
            TextButton::new(defaults_button_rect, "Restore Defaults", ());
        HotkeysPane {
            hotkey_boxes,
            defaults_button,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        for hotkey_box in self.hotkey_boxes.iter() {
            let keycode = state.prefs().hotkey_code(hotkey_box.hotkey());
            hotkey_box.draw(resources, matrix, keycode);
        }

        let enabled = !state.prefs().hotkeys_are_defaults();
        self.defaults_button.draw(resources, matrix, enabled);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PrefsAction> {
        let enabled = !state.prefs().hotkeys_are_defaults();
        if let Some(()) = self.defaults_button.handle_event(event, enabled) {
            state.prefs_mut().set_hotkeys_to_defaults();
            return None;
        }

        let mut listening: Option<Hotkey> = None;
        for hotkey_box in self.hotkey_boxes.iter_mut() {
            match hotkey_box.handle_event(event) {
                Some(HotkeyBoxAction::Listening) => {
                    listening = Some(hotkey_box.hotkey());
                }
                Some(HotkeyBoxAction::Update(keycode)) => {
                    let hotkey = hotkey_box.hotkey();
                    state.prefs_mut().set_hotkey_code(hotkey, keycode);
                }
                None => {}
            }
        }
        if let Some(hotkey) = listening {
            for hotkey_box in self.hotkey_boxes.iter_mut() {
                if hotkey_box.hotkey() != hotkey {
                    hotkey_box.handle_event(&Event::Unfocus);
                }
            }
        }
        return None;
    }
}

//===========================================================================//

pub struct ProfilesPane {
    profile_list: ListView<String>,
    new_button: TextButton<PrefsAction>,
    delete_button: TextButton<PrefsAction>,
}

impl ProfilesPane {
    pub fn new(rect: Rect<i32>, state: &GameState) -> ProfilesPane {
        debug_assert!(state.profile().is_some());
        let profile_list =
            ListView::new(Rect::new(rect.x, rect.y, 300, rect.height),
                          state.profile().unwrap().name(),
                          profile_list_items(state));
        let new_button =
            TextButton::new(Rect::new(rect.right() - 150, rect.y, 150, 40),
                            "New Profile",
                            PrefsAction::NewProfile);
        let delete_button = TextButton::new(Rect::new(rect.right() - 150,
                                                      rect.bottom() - 40,
                                                      150,
                                                      40),
                                            "Delete Profile",
                                            PrefsAction::DeleteProfile);
        ProfilesPane {
            profile_list,
            new_button,
            delete_button,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>,
                state: &GameState) {
        let current_profile_name = state.profile().unwrap().name();
        self.profile_list.draw(resources, matrix, current_profile_name);
        self.new_button.draw(resources, matrix, true);
        self.delete_button.draw(resources, matrix, true);
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut GameState)
                        -> Option<PrefsAction> {
        let current_profile_name = state.profile().unwrap().name();
        if let Some(profile_name) =
            self.profile_list.handle_event(event, current_profile_name)
        {
            return Some(PrefsAction::SwitchProfile(profile_name));
        }
        if let Some(action) = self.new_button.handle_event(event, true) {
            return Some(action);
        }
        if let Some(action) = self.delete_button.handle_event(event, true) {
            return Some(action);
        }
        return None;
    }

    pub fn update_profile_list(&mut self, state: &GameState) {
        self.profile_list.set_items(state.profile().unwrap().name(),
                                    profile_list_items(state));
    }
}

fn profile_list_items(state: &GameState) -> Vec<(String, String)> {
    state
        .profile_names()
        .map(|name| (name.to_string(), name.to_string()))
        .collect()
}

//===========================================================================//
