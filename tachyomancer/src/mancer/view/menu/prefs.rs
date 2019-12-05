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

use super::super::button::{
    Checkbox, HotkeyBox, HotkeyBoxAction, RadioButton, RadioCheckbox,
    Scrollbar, Slider, SliderAction, TextButton, CHECKBOX_HEIGHT,
};
use super::super::paragraph::Paragraph;
use super::credits::credits_paragraphs;
use super::list::{ListIcon, ListView};
use crate::mancer::font::Align;
use crate::mancer::gl::Stencil;
use crate::mancer::gui::{Event, Resources, Sound, Ui, Window, WindowOptions};
use crate::mancer::save::{Hotkey, Prefs, HOTKEY_CATEGORIES};
use crate::mancer::state::GameState;
use cgmath::{Matrix4, Point2};
use num_integer::Roots;
use tachy::geom::{AsFloat, Color4, Rect, RectSize};

//===========================================================================//

const AV_CATEGORY_FRAME_PADDING: i32 = 18;
const AV_CATEGORY_FRAME_SPACING: i32 = 22;
const AV_SLIDER_HEIGHT: i32 = 30;
const AV_SLIDER_SPACING: i32 = 20;
const AV_SLIDER_MARGIN: i32 = 62;
const AV_RESOLUTION_TABLE_MARGIN: i32 = 16;
const AV_RESOLUTION_COLUMN_WIDTH: i32 = 172;
const AV_RESOLUTION_ROW_SPACING: i32 = 6;
const AV_BUTTON_WIDTH: i32 = 200;
const AV_BUTTON_HEIGHT: i32 = 40;
const AV_BUTTON_SPACING: i32 = 24;

const HOTKEY_FRAME_PADDING: i32 = AV_CATEGORY_FRAME_PADDING;
const HOTKEY_CATEGORY_LABEL_FONT_SIZE: f32 = 22.0;
const HOTKEY_CATEGORY_LABEL_STRIDE: i32 = 24;
const HOTKEY_BOX_STRIDE: i32 = 32;
const HOTKEY_CATEGORY_SPACING: i32 = 32;
const HOTKEY_BUTTON_WIDTH: i32 = 200;
const HOTKEY_BUTTON_HEIGHT: i32 = 40;

const CREDITS_FRAME_PADDING: i32 = AV_CATEGORY_FRAME_PADDING;
const CREDITS_PARAGRAPH_SPACING: i32 = 50;
const CREDITS_SCROLLBAR_WIDTH: i32 = 20;
const CREDITS_SCROLLBAR_MARGIN: i32 = 5;

const PANE_BUTTON_SPACING: i32 = 24;
const PANE_BUTTON_WIDTH: i32 = 180;

//===========================================================================//

#[derive(Clone)]
pub enum PrefsAction {
    RebootWindow(WindowOptions),
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
    credits_pane: CreditsPane,
}

impl PrefsView {
    pub fn new(
        rect: Rect<i32>,
        window: &mut Window,
        state: &GameState,
    ) -> PrefsView {
        let num_panes = PANES.len() as i32;
        let pane_button_height = (rect.height + PANE_BUTTON_SPACING)
            / (num_panes + 1)
            - PANE_BUTTON_SPACING;
        let pane_buttons = PANES
            .iter()
            .enumerate()
            .map(|(index, &(pane, label))| {
                let top = rect.y
                    + (index as i32)
                        * (pane_button_height + PANE_BUTTON_SPACING);
                let rect = Rect::new(
                    rect.x,
                    top,
                    PANE_BUTTON_WIDTH,
                    pane_button_height,
                );
                RadioButton::new(rect, label, pane)
            })
            .collect();

        let quit_button_top =
            rect.y + num_panes * (pane_button_height + PANE_BUTTON_SPACING);
        let quit_button_height = rect.height - (quit_button_top - rect.y);
        let quit_button = TextButton::new(
            Rect::new(
                rect.x,
                quit_button_top,
                PANE_BUTTON_WIDTH,
                quit_button_height,
            ),
            "Exit Game",
            PrefsAction::QuitGame,
        );

        let pane_offset = PANE_BUTTON_WIDTH + PANE_BUTTON_SPACING;
        let pane_rect = Rect::new(
            rect.x + pane_offset,
            rect.y,
            rect.width - pane_offset,
            rect.height,
        );
        let audio_video_pane = AudioVideoPane::new(pane_rect, window, state);
        let mut ui = window.ui();
        let hotkeys_pane = HotkeysPane::new(pane_rect);
        let profiles_pane = ProfilesPane::new(pane_rect, &mut ui, state);
        let credits_pane = CreditsPane::new(pane_rect, state.prefs());

        PrefsView {
            current_pane: PrefsPane::AudioVideo,
            pane_buttons,
            quit_button,
            audio_video_pane,
            hotkeys_pane,
            profiles_pane,
            credits_pane,
        }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        debug_assert!(state.profile().is_some());
        for button in self.pane_buttons.iter() {
            button.draw(resources, matrix, &self.current_pane);
        }
        self.quit_button.draw(resources, matrix, true);

        match self.current_pane {
            PrefsPane::AudioVideo => {
                self.audio_video_pane.draw(resources, matrix);
            }
            PrefsPane::Hotkeys => {
                self.hotkeys_pane.draw(resources, matrix, state);
            }
            PrefsPane::Profiles => {
                self.profiles_pane.draw(resources, matrix, state);
            }
            PrefsPane::Credits => {
                self.credits_pane.draw(resources, matrix);
            }
        }
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PrefsAction> {
        debug_assert!(state.profile().is_some());
        if let Some(action) = self.on_pane_event(event, ui, state) {
            return Some(action);
        }

        let mut next_pane: Option<PrefsPane> = None;
        for button in self.pane_buttons.iter_mut() {
            if let Some(pane) = button.on_event(event, ui, &self.current_pane)
            {
                next_pane = Some(pane);
                break;
            }
        }
        if let Some(pane) = next_pane {
            self.on_pane_event(&Event::Unfocus, ui, state);
            self.current_pane = pane;
            ui.request_redraw();
        }
        if let Some(action) = self.quit_button.on_event(event, ui, true) {
            return Some(action);
        }

        return None;
    }

    fn on_pane_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PrefsAction> {
        match self.current_pane {
            PrefsPane::AudioVideo => {
                self.audio_video_pane.on_event(event, ui, state)
            }
            PrefsPane::Hotkeys => self.hotkeys_pane.on_event(event, ui, state),
            PrefsPane::Profiles => {
                self.profiles_pane.on_event(event, ui, state)
            }
            PrefsPane::Credits => self.credits_pane.on_event(event, ui),
        }
    }

    pub fn update_profile_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.profiles_pane.update_profile_list(ui, state);
    }
}

//===========================================================================//

pub struct AudioVideoPane {
    category_frames: Vec<Rect<f32>>,
    antialias_checkbox: Checkbox,
    fullscreen_checkbox: Checkbox,
    resolution_checkboxes: Vec<RadioCheckbox<Option<RectSize<i32>>>>,
    sound_volume_slider: Slider,
    music_volume_slider: Slider,
    apply_button: TextButton<()>,
    revert_button: TextButton<()>,
    current_window_options: WindowOptions,
    new_window_options: WindowOptions,
}

impl AudioVideoPane {
    pub fn new(
        rect: Rect<i32>,
        window: &Window,
        state: &GameState,
    ) -> AudioVideoPane {
        let mut category_frames = Vec::new();
        let mut frame_top = rect.y;
        let mut top = frame_top + AV_CATEGORY_FRAME_PADDING;
        let left = rect.x + AV_CATEGORY_FRAME_PADDING;
        let right = rect.right() - AV_CATEGORY_FRAME_PADDING;

        // Audio section:
        let music_volume_slider = Slider::new(
            Rect::new(
                rect.x + AV_CATEGORY_FRAME_PADDING + AV_SLIDER_MARGIN,
                top,
                right - left - 2 * AV_SLIDER_MARGIN,
                AV_SLIDER_HEIGHT,
            ),
            state.prefs().music_volume_percent(),
            "Music".to_string(),
        );
        top += AV_SLIDER_HEIGHT + AV_SLIDER_SPACING;
        let sound_volume_slider = Slider::new(
            Rect::new(
                left + AV_SLIDER_MARGIN,
                top,
                right - left - 2 * AV_SLIDER_MARGIN,
                AV_SLIDER_HEIGHT,
            ),
            state.prefs().sound_volume_percent(),
            "Sound".to_string(),
        );
        top += AV_SLIDER_HEIGHT + AV_CATEGORY_FRAME_PADDING;
        category_frames.push(
            Rect::new(rect.x, frame_top, rect.width, top - frame_top).as_f32(),
        );
        top += AV_CATEGORY_FRAME_SPACING;
        frame_top = top;

        // Video section:
        top += AV_CATEGORY_FRAME_PADDING;
        let fullscreen_checkbox =
            Checkbox::new(Point2::new(left, top), "Fullscreen".to_string());
        let antialias_checkbox = Checkbox::new(
            Point2::new(left + AV_BUTTON_WIDTH + AV_BUTTON_SPACING, top),
            "Antialiasing".to_string(),
        );
        top += CHECKBOX_HEIGHT;
        let button_top =
            rect.bottom() - AV_CATEGORY_FRAME_PADDING - AV_BUTTON_HEIGHT;
        let resolution_checkboxes = {
            let mut resolutions = vec![("Native".to_string(), None)];
            for &res in window.possible_resolutions().iter() {
                let label = format!("{}x{}", res.width, res.height);
                resolutions.push((label, Some(res)));
            }
            let max_rows = (button_top - top - 2 * AV_RESOLUTION_TABLE_MARGIN
                + AV_RESOLUTION_ROW_SPACING)
                / (CHECKBOX_HEIGHT + AV_RESOLUTION_ROW_SPACING);
            let max_cols = (right - left) / AV_RESOLUTION_COLUMN_WIDTH;
            resolutions.truncate((max_rows as usize) * (max_cols as usize));
            let num_resolutions = resolutions.len() as i32;
            let num_rows = if max_rows < max_cols {
                num_resolutions.sqrt().min(max_rows)
            } else {
                let num_cols = num_resolutions.sqrt().min(max_cols);
                (num_resolutions + num_cols - 1) / num_cols
            };
            let table_height = num_rows
                * (CHECKBOX_HEIGHT + AV_RESOLUTION_ROW_SPACING)
                - AV_RESOLUTION_ROW_SPACING;
            let table_top = top + ((button_top - top) - table_height) / 2;
            resolutions
                .into_iter()
                .enumerate()
                .map(|(index, (label, res))| {
                    let row = (index as i32) % num_rows;
                    let col = (index as i32) / num_rows;
                    let x = left + col * AV_RESOLUTION_COLUMN_WIDTH;
                    let y = table_top
                        + row * (CHECKBOX_HEIGHT + AV_RESOLUTION_ROW_SPACING);
                    RadioCheckbox::new(Point2::new(x, y), label, res)
                })
                .collect()
        };
        let apply_button_rect =
            Rect::new(left, button_top, AV_BUTTON_WIDTH, AV_BUTTON_HEIGHT);
        let apply_button = TextButton::new(apply_button_rect, "Apply", ());
        let revert_button_rect = Rect::new(
            left + AV_BUTTON_WIDTH + AV_BUTTON_SPACING,
            button_top,
            AV_BUTTON_WIDTH,
            AV_BUTTON_HEIGHT,
        );
        let revert_button = TextButton::new(revert_button_rect, "Revert", ());
        category_frames.push(
            Rect::new(
                rect.x,
                frame_top,
                rect.width,
                rect.bottom() - frame_top,
            )
            .as_f32(),
        );

        AudioVideoPane {
            category_frames,
            antialias_checkbox,
            fullscreen_checkbox,
            resolution_checkboxes,
            sound_volume_slider,
            music_volume_slider,
            apply_button,
            revert_button,
            current_window_options: window.options().clone(),
            new_window_options: window.options().clone(),
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        for rect in self.category_frames.iter() {
            resources.shaders().ui().draw_bubble(
                matrix,
                rect,
                &Color4::CYAN1,
                &Color4::ORANGE1,
                &Color4::PURPLE0_TRANSLUCENT,
            );
        }

        self.antialias_checkbox.draw(
            resources,
            matrix,
            self.new_window_options.antialiasing,
            true,
        );
        self.fullscreen_checkbox.draw(
            resources,
            matrix,
            self.new_window_options.fullscreen,
            true,
        );
        for button in self.resolution_checkboxes.iter() {
            button.draw(
                resources,
                matrix,
                &self.new_window_options.resolution,
            );
        }
        self.sound_volume_slider.draw(resources, matrix);
        self.music_volume_slider.draw(resources, matrix);

        let enabled = self.new_window_options != self.current_window_options;
        self.apply_button.draw(resources, matrix, enabled);
        self.revert_button.draw(resources, matrix, enabled);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PrefsAction> {
        if let &Event::Unfocus = event {
            self.new_window_options = self.current_window_options.clone();
        }

        let antialiasing = self.new_window_options.antialiasing;
        if let Some(checked) =
            self.antialias_checkbox.on_event(event, ui, antialiasing, true)
        {
            self.new_window_options.antialiasing = checked;
        }

        if let Some(checked) = self.fullscreen_checkbox.on_event(
            event,
            ui,
            self.new_window_options.fullscreen,
            true,
        ) {
            self.new_window_options.fullscreen = checked;
        }

        let resolution = self.new_window_options.resolution;
        for button in self.resolution_checkboxes.iter_mut() {
            if let Some(new_res) = button.on_event(event, ui, &resolution) {
                self.new_window_options.resolution = new_res;
            }
        }

        match self.sound_volume_slider.on_event(event, ui) {
            Some(SliderAction::Update(volume)) => {
                state.prefs_mut().set_sound_volume_percent(volume);
                ui.audio().set_sound_volume_percent(volume);
            }
            Some(SliderAction::Release) => {
                ui.audio().play_sound(Sound::Beep);
            }
            None => {}
        }

        match self.music_volume_slider.on_event(event, ui) {
            Some(SliderAction::Update(volume)) => {
                state.prefs_mut().set_music_volume_percent(volume);
                ui.audio().set_music_volume_percent(volume);
            }
            Some(SliderAction::Release) => {}
            None => {}
        }

        let enabled = self.new_window_options != self.current_window_options;
        if let Some(()) = self.revert_button.on_event(event, ui, enabled) {
            self.new_window_options = self.current_window_options.clone();
        }
        if let Some(()) = self.apply_button.on_event(event, ui, enabled) {
            let prefs = state.prefs_mut();
            prefs.set_antialiasing(self.new_window_options.antialiasing);
            prefs.set_fullscreen(self.new_window_options.fullscreen);
            prefs.set_resolution(self.new_window_options.resolution);
            let options = self.new_window_options.clone();
            return Some(PrefsAction::RebootWindow(options));
        }
        return None;
    }
}

//===========================================================================//

pub struct HotkeysPane {
    rect: Rect<i32>,
    category_labels: Vec<((f32, f32), &'static str)>,
    hotkey_boxes: Vec<(Hotkey, HotkeyBox)>,
    defaults_button: TextButton<()>,
}

impl HotkeysPane {
    pub fn new(rect: Rect<i32>) -> HotkeysPane {
        let mut left = rect.x + HOTKEY_FRAME_PADDING;
        let mut top = rect.y + HOTKEY_FRAME_PADDING;
        let mut category_labels = Vec::new();
        let mut hotkey_boxes = Vec::new();
        for &(name, hotkeys) in HOTKEY_CATEGORIES.iter() {
            let section_height = HOTKEY_CATEGORY_LABEL_STRIDE
                + HOTKEY_BOX_STRIDE * (hotkeys.len() as i32);
            if rect.bottom() - HOTKEY_FRAME_PADDING - top < section_height {
                left = rect.x + rect.width / 2;
                top = rect.y + HOTKEY_FRAME_PADDING;
            }
            category_labels.push(((left as f32, top as f32), name));
            top += HOTKEY_CATEGORY_LABEL_STRIDE;
            for &hotkey in hotkeys.iter() {
                hotkey_boxes.push((
                    hotkey,
                    HotkeyBox::new(
                        Point2::new(left, top),
                        hotkey.name().to_string(),
                    ),
                ));
                top += HOTKEY_BOX_STRIDE;
            }
            top += HOTKEY_CATEGORY_SPACING;
        }
        let defaults_button_rect = Rect::new(
            rect.right() - HOTKEY_FRAME_PADDING - HOTKEY_BUTTON_WIDTH,
            rect.bottom() - HOTKEY_FRAME_PADDING - HOTKEY_BUTTON_HEIGHT,
            HOTKEY_BUTTON_WIDTH,
            HOTKEY_BUTTON_HEIGHT,
        );
        let defaults_button =
            TextButton::new(defaults_button_rect, "Restore Defaults", ());
        HotkeysPane { rect, category_labels, hotkey_boxes, defaults_button }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        resources.shaders().ui().draw_bubble(
            matrix,
            &self.rect.as_f32(),
            &Color4::CYAN1,
            &Color4::ORANGE1,
            &Color4::PURPLE0_TRANSLUCENT,
        );
        for &(position, label) in self.category_labels.iter() {
            resources.fonts().bold().draw(
                matrix,
                HOTKEY_CATEGORY_LABEL_FONT_SIZE,
                Align::TopLeft,
                position,
                label,
            );
        }

        for &(hotkey, ref hotkey_box) in self.hotkey_boxes.iter() {
            let code = state.prefs().hotkey_code(hotkey);
            hotkey_box.draw(resources, matrix, Some(code));
        }

        let enabled = !state.prefs().hotkeys_are_defaults();
        self.defaults_button.draw(resources, matrix, enabled);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PrefsAction> {
        let enabled = !state.prefs().hotkeys_are_defaults();
        if self.defaults_button.on_event(event, ui, enabled).is_some() {
            state.prefs_mut().set_hotkeys_to_defaults();
            return None;
        }

        let mut listening: Option<Hotkey> = None;
        for &mut (hotkey, ref mut hotkey_box) in self.hotkey_boxes.iter_mut() {
            match hotkey_box.on_event(event, ui) {
                Some(HotkeyBoxAction::Listening) => {
                    listening = Some(hotkey);
                }
                Some(HotkeyBoxAction::Set(code)) => {
                    state.prefs_mut().set_hotkey_code(hotkey, code);
                }
                Some(HotkeyBoxAction::Clear) => {}
                None => {}
            }
        }
        if let Some(listening_hotkey) = listening {
            for &mut (hotkey, ref mut hotkey_box) in
                self.hotkey_boxes.iter_mut()
            {
                if hotkey != listening_hotkey {
                    hotkey_box.on_event(&Event::Unfocus, ui);
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
    pub fn new(
        rect: Rect<i32>,
        ui: &mut Ui,
        state: &GameState,
    ) -> ProfilesPane {
        debug_assert!(state.profile().is_some());
        let profile_list = ListView::new(
            Rect::new(rect.x, rect.y, 300, rect.height),
            ui,
            profile_list_items(state),
            state.profile().unwrap().name(),
        );
        let new_button = TextButton::new(
            Rect::new(rect.right() - 150, rect.y, 150, 40),
            "New Profile",
            PrefsAction::NewProfile,
        );
        let delete_button = TextButton::new(
            Rect::new(rect.right() - 150, rect.bottom() - 40, 150, 40),
            "Delete Profile",
            PrefsAction::DeleteProfile,
        );
        ProfilesPane { profile_list, new_button, delete_button }
    }

    pub fn draw(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        state: &GameState,
    ) {
        let current_profile_name = state.profile().unwrap().name();
        self.profile_list.draw(resources, matrix, current_profile_name);
        self.new_button.draw(resources, matrix, true);
        self.delete_button.draw(resources, matrix, true);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        state: &mut GameState,
    ) -> Option<PrefsAction> {
        let current_profile_name = state.profile().unwrap().name();
        if let Some(profile_name) =
            self.profile_list.on_event(event, ui, current_profile_name)
        {
            return Some(PrefsAction::SwitchProfile(profile_name));
        }
        if let Some(action) = self.new_button.on_event(event, ui, true) {
            return Some(action);
        }
        if let Some(action) = self.delete_button.on_event(event, ui, true) {
            return Some(action);
        }
        return None;
    }

    pub fn update_profile_list(&mut self, ui: &mut Ui, state: &GameState) {
        self.profile_list.set_items(
            ui,
            profile_list_items(state),
            state.profile().unwrap().name(),
        );
    }
}

fn profile_list_items(
    state: &GameState,
) -> Vec<(String, String, bool, Option<ListIcon>)> {
    state
        .profile_names()
        .map(|name| (name.to_string(), name.to_string(), false, None))
        .collect()
}

//===========================================================================//

pub struct CreditsPane {
    rect: Rect<i32>,
    frame_rect: Rect<f32>,
    paragraphs: Vec<Paragraph>,
    scrollbar: Scrollbar,
}

impl CreditsPane {
    pub fn new(rect: Rect<i32>, prefs: &Prefs) -> CreditsPane {
        let frame_width =
            rect.width - (CREDITS_SCROLLBAR_MARGIN + CREDITS_SCROLLBAR_WIDTH);
        let paragraph_width = (frame_width - 2 * CREDITS_FRAME_PADDING) as f32;
        let paragraphs = credits_paragraphs(paragraph_width, prefs);
        let mut total_height = 0;
        for paragraph in paragraphs.iter() {
            if total_height > 0 {
                total_height += CREDITS_PARAGRAPH_SPACING;
            }
            total_height += paragraph.height().ceil() as i32;
        }
        total_height += 2 * CREDITS_FRAME_PADDING;
        let scrollbar_rect = Rect::new(
            rect.right() - CREDITS_SCROLLBAR_WIDTH,
            rect.y,
            CREDITS_SCROLLBAR_WIDTH,
            rect.height,
        );
        let scrollbar = Scrollbar::new(scrollbar_rect, total_height);
        let frame_rect =
            Rect::new(rect.x, rect.y, frame_width, rect.height).as_f32();
        CreditsPane { rect, frame_rect, paragraphs, scrollbar }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let stencil = Stencil::new();
        resources.shaders().ui().draw_bubble(
            matrix,
            &self.frame_rect,
            &Color4::CYAN1,
            &Color4::ORANGE1,
            &Color4::PURPLE0_TRANSLUCENT,
        );
        stencil.enable_clipping();
        let left = (self.rect.x + CREDITS_FRAME_PADDING) as f32;
        let mut top = (self.rect.y + CREDITS_FRAME_PADDING
            - self.scrollbar.scroll_top()) as f32;
        for paragraph in self.paragraphs.iter() {
            let bottom = top + paragraph.height().ceil();
            if bottom > self.frame_rect.y && top < self.frame_rect.bottom() {
                paragraph.draw(resources, matrix, (left, top));
            }
            top = bottom + (CREDITS_PARAGRAPH_SPACING as f32);
        }
        stencil.disable();
        self.scrollbar.draw(resources, matrix);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<PrefsAction> {
        self.scrollbar.on_event(event, ui);
        match event {
            Event::Scroll(scroll) if self.rect.contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y, ui);
            }
            _ => {}
        }
        None
    }
}

//===========================================================================//
