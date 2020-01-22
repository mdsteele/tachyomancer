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

use super::button::{
    HotkeyBox, HotkeyBoxAction, RadioCheckbox, TextBox, TextButton,
    CHECKBOX_HEIGHT, HOTKEY_BOX_HEIGHT, HOTKEY_BOX_WIDTH, TEXT_BUTTON_FONT,
    TEXT_BUTTON_FONT_SIZE,
};
use super::graph::ScoreGraph;
use super::paragraph::Paragraph;
use crate::mancer::gui::{Cursor, Event, Keycode, Resources, Ui};
use crate::mancer::save::Prefs;
use cgmath::{Matrix4, Point2};
use tachy::geom::{AsFloat, Color4, Rect, RectSize};
use tachy::save::{HotkeyCode, Puzzle, ScoreCurve, WireSize};

//===========================================================================//

const DIALOG_COLOR_1: Color4 = Color4::CYAN4;
const DIALOG_COLOR_2: Color4 = Color4::ORANGE5;
const DIALOG_COLOR_3: Color4 = Color4::PURPLE0_TRANSLUCENT;

const BUTTON_HEIGHT: i32 = 40;
const BUTTON_INNER_MARGIN: i32 = 10;
const BUTTON_MIN_WIDTH: i32 = 80;
const BUTTON_SPACING: i32 = 14;
const BUTTON_TOP_MARGIN: i32 = 16;

const RADIO_CHECKBOX_TOP_MARGIN: i32 = 16;
const RADIO_CHECKBOX_SPACING_HORZ: i32 = 16;
const RADIO_CHECKBOX_SPACING_VERT: i32 = 6;
const RADIO_CHECKBOX_WIDTH: i32 = 96;

const SCORE_GRAPH_WIDTH: i32 = 250;
const SCORE_GRAPH_HEIGHT: i32 = SCORE_GRAPH_WIDTH;
const SCORE_GRAPH_TOP_MARGIN: i32 = BUTTON_TOP_MARGIN;

const FONT_SIZE: f32 = 20.0;
const HOTKEY_BOX_TOP_MARGIN: i32 = 16;
const LINE_HEIGHT: f32 = 24.0;
const MARGIN: i32 = 24;
const MAX_PARAGRAPH_WIDTH: f32 = 600.0;
const TEXTBOX_HEIGHT: i32 = 32;
const TEXTBOX_TOP_MARGIN: i32 = 16;

//===========================================================================//

pub enum DialogAction<T> {
    Cancel,
    Value(T),
}

//===========================================================================//

pub struct ButtonDialogBox<T> {
    rect: Rect<i32>,
    paragraph: Paragraph,
    buttons: Vec<TextButton<T>>,
}

impl<T: Clone> ButtonDialogBox<T> {
    pub fn new(
        window_size: RectSize<i32>,
        prefs: &Prefs,
        format: &str,
        buttons: &[(&str, T, Option<Keycode>)],
    ) -> ButtonDialogBox<T> {
        let paragraph = Paragraph::compile(
            FONT_SIZE,
            LINE_HEIGHT,
            MAX_PARAGRAPH_WIDTH,
            prefs,
            format,
        );

        let mut width = 0;
        let buttons: Vec<(&str, T, Option<Keycode>, i32, i32)> = buttons
            .iter()
            .map(|&(label, ref value, key)| {
                let label_width =
                    TEXT_BUTTON_FONT.str_width(TEXT_BUTTON_FONT_SIZE, label);
                let button_width = BUTTON_MIN_WIDTH.max(
                    (label_width.ceil() as i32) + 2 * BUTTON_INNER_MARGIN,
                );
                if width > 0 {
                    width += BUTTON_SPACING;
                }
                width += button_width;
                (label, value.clone(), key, width, button_width)
            })
            .collect();
        width = width.max(paragraph.width().ceil() as i32);
        width += 2 * MARGIN;

        let button_top =
            MARGIN + (paragraph.height().ceil() as i32) + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new(
            (window_size.width - width) / 2,
            (window_size.height - height) / 2,
            width,
            height,
        );

        let buttons = buttons
            .into_iter()
            .map(|(label, value, key, button_offset, button_width)| {
                let button_rect = Rect::new(
                    rect.right() - MARGIN - button_offset,
                    rect.y + button_top,
                    button_width,
                    BUTTON_HEIGHT,
                );
                TextButton::new_with_key(button_rect, label, value, key)
            })
            .collect();

        ButtonDialogBox { rect, paragraph, buttons }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(
            &matrix,
            &rect,
            &DIALOG_COLOR_1,
            &DIALOG_COLOR_2,
            &DIALOG_COLOR_3,
        );

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        for button in self.buttons.iter() {
            button.draw(resources, matrix, true);
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) -> Option<T> {
        for button in self.buttons.iter_mut() {
            if let Some(value) = button.on_event(event, ui, true) {
                return Some(value);
            }
        }
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//

pub struct HotkeyDialogBox {
    rect: Rect<i32>,
    paragraph: Paragraph,
    hotkey_box: HotkeyBox,
    opt_code: Option<HotkeyCode>,
    ok_button: TextButton<()>,
    cancel_button: TextButton<()>,
}

impl HotkeyDialogBox {
    pub fn new(
        window_size: RectSize<i32>,
        prefs: &Prefs,
        format: &str,
        initial: Option<HotkeyCode>,
    ) -> HotkeyDialogBox {
        let paragraph = Paragraph::compile(
            FONT_SIZE,
            LINE_HEIGHT,
            MAX_PARAGRAPH_WIDTH,
            prefs,
            format,
        );
        let width = (paragraph.width().ceil() as i32)
            .max(2 * BUTTON_MIN_WIDTH + BUTTON_SPACING)
            .max(HOTKEY_BOX_WIDTH)
            + 2 * MARGIN;

        let hotkey_box_top = MARGIN
            + (paragraph.height().ceil() as i32)
            + HOTKEY_BOX_TOP_MARGIN;
        let button_top =
            hotkey_box_top + HOTKEY_BOX_HEIGHT + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new(
            (window_size.width - width) / 2,
            (window_size.height - height) / 2,
            width,
            height,
        );

        let hotkey_box_left_top = Point2::new(
            rect.x + (rect.width - HOTKEY_BOX_WIDTH) / 2,
            rect.y + hotkey_box_top,
        );
        let hotkey_box = HotkeyBox::new(hotkey_box_left_top, String::new());

        let ok_button_rect = Rect::new(
            rect.right() - MARGIN - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let ok_button = TextButton::new_with_key(
            ok_button_rect,
            "OK",
            (),
            Some(Keycode::Return),
        );

        let cancel_button_rect = Rect::new(
            ok_button_rect.x - BUTTON_SPACING - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let cancel_button = TextButton::new_with_key(
            cancel_button_rect,
            "Cancel",
            (),
            Some(Keycode::Escape),
        );

        HotkeyDialogBox {
            rect,
            paragraph,
            hotkey_box,
            opt_code: initial,
            ok_button,
            cancel_button,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(
            &matrix,
            &rect,
            &DIALOG_COLOR_1,
            &DIALOG_COLOR_2,
            &DIALOG_COLOR_3,
        );

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        self.hotkey_box.draw(resources, matrix, self.opt_code);
        self.ok_button.draw(resources, matrix, true);
        self.cancel_button.draw(resources, matrix, true);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<DialogAction<Option<HotkeyCode>>> {
        match self.hotkey_box.on_event(event, ui) {
            Some(HotkeyBoxAction::Listening) => {}
            Some(HotkeyBoxAction::Set(code)) => {
                self.opt_code = Some(code);
                ui.request_redraw();
            }
            Some(HotkeyBoxAction::Clear) => {
                self.opt_code = None;
                ui.request_redraw();
            }
            None => {}
        }
        if let Some(()) = self.ok_button.on_event(event, ui, true) {
            return Some(DialogAction::Value(self.opt_code));
        }
        if let Some(()) = self.cancel_button.on_event(event, ui, true) {
            return Some(DialogAction::Cancel);
        }
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//

pub struct ScoreGraphDialogBox<T> {
    rect: Rect<i32>,
    paragraph: Paragraph,
    graph: ScoreGraph,
    buttons: Vec<TextButton<T>>,
}

impl<T: Clone> ScoreGraphDialogBox<T> {
    pub fn new(
        window_size: RectSize<i32>,
        prefs: &Prefs,
        format: &str,
        puzzle: Puzzle,
        local_scores: &ScoreCurve,
        hilight_score: (i32, u32),
        buttons: &[(&str, T, Option<Keycode>)],
    ) -> ScoreGraphDialogBox<T> {
        let paragraph = Paragraph::compile(
            FONT_SIZE,
            LINE_HEIGHT,
            MAX_PARAGRAPH_WIDTH,
            prefs,
            format,
        );

        let mut width = 0;
        let buttons: Vec<(&str, T, Option<Keycode>, i32, i32)> = buttons
            .iter()
            .map(|&(label, ref value, key)| {
                let label_width =
                    TEXT_BUTTON_FONT.str_width(TEXT_BUTTON_FONT_SIZE, label);
                let button_width = BUTTON_MIN_WIDTH.max(
                    (label_width.ceil() as i32) + 2 * BUTTON_INNER_MARGIN,
                );
                if width > 0 {
                    width += BUTTON_SPACING;
                }
                width += button_width;
                (label, value.clone(), key, width, button_width)
            })
            .collect();
        width = width.max(SCORE_GRAPH_WIDTH);
        width = width.max(paragraph.width().ceil() as i32);
        width += 2 * MARGIN;

        let graph_top = MARGIN
            + (paragraph.height().ceil() as i32)
            + SCORE_GRAPH_TOP_MARGIN;
        let button_top = graph_top + SCORE_GRAPH_HEIGHT + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new(
            (window_size.width - width) / 2,
            (window_size.height - height) / 2,
            width,
            height,
        );

        let graph_rect = Rect::new(
            rect.x + (width - SCORE_GRAPH_WIDTH) / 2,
            rect.y + graph_top,
            SCORE_GRAPH_WIDTH,
            SCORE_GRAPH_HEIGHT,
        );
        let graph = ScoreGraph::new(
            window_size,
            graph_rect.as_f32(),
            puzzle,
            local_scores,
            Some(hilight_score),
        );

        let buttons = buttons
            .into_iter()
            .map(|(label, value, key, button_offset, button_width)| {
                let button_rect = Rect::new(
                    rect.right() - MARGIN - button_offset,
                    rect.y + button_top,
                    button_width,
                    BUTTON_HEIGHT,
                );
                TextButton::new_with_key(button_rect, label, value, key)
            })
            .collect();

        ScoreGraphDialogBox { rect, paragraph, graph, buttons }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(
            &matrix,
            &rect,
            &DIALOG_COLOR_1,
            &DIALOG_COLOR_2,
            &DIALOG_COLOR_3,
        );

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        self.graph.draw(resources, matrix);

        for button in self.buttons.iter() {
            button.draw(resources, matrix, true);
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui) -> Option<T> {
        for button in self.buttons.iter_mut() {
            if let Some(value) = button.on_event(event, ui, true) {
                return Some(value);
            }
        }
        self.graph.on_event(event, ui);
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//

pub struct TextDialogBox {
    rect: Rect<i32>,
    paragraph: Paragraph,
    textbox: TextBox,
    ok_button: TextButton<()>,
    cancel_button: TextButton<()>,
}

impl TextDialogBox {
    pub fn new(
        window_size: RectSize<i32>,
        prefs: &Prefs,
        format: &str,
        initial: &str,
        max_len: usize,
    ) -> TextDialogBox {
        let paragraph = Paragraph::compile(
            FONT_SIZE,
            LINE_HEIGHT,
            MAX_PARAGRAPH_WIDTH,
            prefs,
            format,
        );
        let textbox_width = (paragraph.width().ceil() as i32)
            .max(2 * BUTTON_MIN_WIDTH + BUTTON_SPACING);
        let width = textbox_width + 2 * MARGIN;

        let textbox_top =
            MARGIN + (paragraph.height().ceil() as i32) + TEXTBOX_TOP_MARGIN;
        let button_top = textbox_top + TEXTBOX_HEIGHT + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new(
            (window_size.width - width) / 2,
            (window_size.height - height) / 2,
            width,
            height,
        );

        let textbox_rect = Rect::new(
            rect.x + MARGIN,
            rect.y + textbox_top,
            textbox_width,
            TEXTBOX_HEIGHT,
        );
        let textbox = TextBox::new(textbox_rect, initial, max_len);

        let ok_button_rect = Rect::new(
            rect.right() - MARGIN - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let ok_button = TextButton::new_with_key(
            ok_button_rect,
            "OK",
            (),
            Some(Keycode::Return),
        );

        let cancel_button_rect = Rect::new(
            ok_button_rect.x - BUTTON_SPACING - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let cancel_button = TextButton::new_with_key(
            cancel_button_rect,
            "Cancel",
            (),
            Some(Keycode::Escape),
        );

        TextDialogBox { rect, paragraph, textbox, ok_button, cancel_button }
    }

    pub fn draw<F>(
        &self,
        resources: &Resources,
        matrix: &Matrix4<f32>,
        is_valid: F,
    ) where
        F: Fn(&str) -> bool,
    {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(
            &matrix,
            &rect,
            &DIALOG_COLOR_1,
            &DIALOG_COLOR_2,
            &DIALOG_COLOR_3,
        );

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        self.textbox.draw(resources, matrix);
        let valid = is_valid(&self.textbox.string());
        self.ok_button.draw(resources, matrix, valid);
        self.cancel_button.draw(resources, matrix, true);
    }

    pub fn on_event<F>(
        &mut self,
        event: &Event,
        ui: &mut Ui,
        is_valid: F,
    ) -> Option<DialogAction<String>>
    where
        F: Fn(&str) -> bool,
    {
        self.textbox.on_event(event, ui);
        let string = self.textbox.string();
        let valid = is_valid(string);
        if let Some(()) = self.ok_button.on_event(event, ui, valid) {
            return Some(DialogAction::Value(string.to_string()));
        }
        if let Some(()) = self.cancel_button.on_event(event, ui, true) {
            return Some(DialogAction::Cancel);
        }
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//

const WIRE_SIZES: &[WireSize] = &[
    WireSize::One,
    WireSize::Two,
    WireSize::Four,
    WireSize::Eight,
    WireSize::Sixteen,
];

pub struct WireSizeDialogBox {
    rect: Rect<i32>,
    paragraph: Paragraph,
    radio_checkboxes: Vec<RadioCheckbox<WireSize>>,
    size: WireSize,
    ok_button: TextButton<()>,
    cancel_button: TextButton<()>,
}

impl WireSizeDialogBox {
    pub fn new(
        window_size: RectSize<i32>,
        prefs: &Prefs,
        format: &str,
        initial: WireSize,
    ) -> WireSizeDialogBox {
        let paragraph = Paragraph::compile(
            FONT_SIZE,
            LINE_HEIGHT,
            MAX_PARAGRAPH_WIDTH,
            prefs,
            format,
        );

        let radio_checkboxes_cols = 2;
        let radio_checkboxes_rows =
            ((WIRE_SIZES.len() as i32) + radio_checkboxes_cols - 1)
                / radio_checkboxes_cols;

        let width = (paragraph.width().ceil() as i32)
            .max(2 * BUTTON_MIN_WIDTH + BUTTON_SPACING)
            .max(
                RADIO_CHECKBOX_WIDTH * radio_checkboxes_cols
                    + RADIO_CHECKBOX_SPACING_HORZ
                        * (radio_checkboxes_cols - 1),
            )
            + 2 * MARGIN;

        let radio_checkboxes_top = MARGIN
            + (paragraph.height().ceil() as i32)
            + RADIO_CHECKBOX_TOP_MARGIN;
        let button_top = radio_checkboxes_top
            + CHECKBOX_HEIGHT * radio_checkboxes_rows
            + RADIO_CHECKBOX_SPACING_VERT * (radio_checkboxes_rows - 1)
            + BUTTON_TOP_MARGIN;
        let height = button_top + BUTTON_HEIGHT + MARGIN;

        let rect = Rect::new(
            (window_size.width - width) / 2,
            (window_size.height - height) / 2,
            width,
            height,
        );

        let mut radio_checkboxes = Vec::new();
        let radio_checkboxes_left = (rect.width
            - (RADIO_CHECKBOX_WIDTH * radio_checkboxes_cols
                + RADIO_CHECKBOX_SPACING_HORZ * (radio_checkboxes_cols - 1)))
            / 2;
        for (index, &size) in WIRE_SIZES.iter().enumerate() {
            let col = (index as i32) / radio_checkboxes_rows;
            let row = (index as i32) % radio_checkboxes_rows;
            radio_checkboxes.push(RadioCheckbox::new(
                Point2::new(
                    rect.x
                        + radio_checkboxes_left
                        + col
                            * (RADIO_CHECKBOX_WIDTH
                                + RADIO_CHECKBOX_SPACING_HORZ),
                    rect.y
                        + radio_checkboxes_top
                        + row
                            * (CHECKBOX_HEIGHT + RADIO_CHECKBOX_SPACING_VERT),
                ),
                format!("{}-bit", size.num_bits()),
                size,
            ));
        }

        let ok_button_rect = Rect::new(
            rect.right() - MARGIN - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let ok_button = TextButton::new_with_key(
            ok_button_rect,
            "OK",
            (),
            Some(Keycode::Return),
        );

        let cancel_button_rect = Rect::new(
            ok_button_rect.x - BUTTON_SPACING - BUTTON_MIN_WIDTH,
            rect.y + button_top,
            BUTTON_MIN_WIDTH,
            BUTTON_HEIGHT,
        );
        let cancel_button = TextButton::new_with_key(
            cancel_button_rect,
            "Cancel",
            (),
            Some(Keycode::Escape),
        );

        WireSizeDialogBox {
            rect,
            paragraph,
            radio_checkboxes,
            size: initial,
            ok_button,
            cancel_button,
        }
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let rect = self.rect.as_f32();
        resources.shaders().ui().draw_dialog(
            &matrix,
            &rect,
            &DIALOG_COLOR_1,
            &DIALOG_COLOR_2,
            &DIALOG_COLOR_3,
        );

        let left = (self.rect.x + MARGIN) as f32;
        let top = (self.rect.y + MARGIN) as f32;
        self.paragraph.draw(resources, matrix, (left, top));

        for radio_checkbox in self.radio_checkboxes.iter() {
            radio_checkbox.draw(resources, matrix, &self.size);
        }
        self.ok_button.draw(resources, matrix, true);
        self.cancel_button.draw(resources, matrix, true);
    }

    pub fn on_event(
        &mut self,
        event: &Event,
        ui: &mut Ui,
    ) -> Option<DialogAction<WireSize>> {
        for checkbox in self.radio_checkboxes.iter_mut() {
            if let Some(size) = checkbox.on_event(event, ui, &self.size) {
                self.size = size;
                ui.request_redraw();
            }
        }
        if let Some(()) = self.ok_button.on_event(event, ui, true) {
            return Some(DialogAction::Value(self.size));
        }
        if let Some(()) = self.cancel_button.on_event(event, ui, true) {
            return Some(DialogAction::Cancel);
        }
        if event.is_mouse() {
            ui.cursor().request(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//
