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

use super::super::button::Scrollbar;
use super::super::chip::ChipModel;
use super::tray::TraySlide;
use cgmath::{Deg, Matrix4, Point2, vec2};
use tachy::font::Align;
use tachy::geom::{AsFloat, Color4, MatrixExt, Orientation, Rect, RectSize};
use tachy::gl::Stencil;
use tachy::gui::{Cursor, Event, Resources, Ui};
use tachy::save::{CHIP_CATEGORIES, ChipType, Puzzle};
use tachy::shader::UiShader;

//===========================================================================//

const CATEGORY_LABEL_HEIGHT: i32 = 30;
const CATEGORY_LABEL_FONT_SIZE: f32 = 20.0;

const PART_WIDTH: i32 = 56;
const PART_HEIGHT: i32 = 56;
const PART_INNER_MARGIN: i32 = 4;
const PART_SPACING: i32 = 0;

const SCROLLBAR_MARGIN: i32 = 8;
const SCROLLBAR_WIDTH: i32 = 16;

const TRAY_FLIP_HORZ: bool = false;
const TRAY_INNER_MARGIN: i32 = 16;
const TRAY_OUTER_MARGIN: i32 = 32;
const TRAY_TAB_FONT_SIZE: f32 = 16.0;
const TRAY_TAB_HEIGHT: f32 = 60.0;
const TRAY_TAB_TEXT: &str = "PARTS";

//===========================================================================//

#[derive(Clone, Copy, Debug)]
pub enum PartsAction {
    Grab(ChipType, Point2<i32>),
    Drop,
}

//===========================================================================//

pub struct PartsTray {
    rect: Rect<i32>,
    category_labels: Vec<CategoryLabel>,
    parts: Vec<Part>,
    scrollbar: Scrollbar,
    slide: TraySlide,
    // TODO: tooltip when hovering over part
}

impl PartsTray {
    pub fn new(window_size: RectSize<i32>, current_puzzle: Puzzle)
               -> PartsTray {
        let num_columns = if window_size.width < 1000 {
            2
        } else if window_size.width < 1200 {
            3
        } else {
            4
        };
        let tray_width = 2 * TRAY_INNER_MARGIN +
            num_columns * (PART_WIDTH + PART_SPACING) -
            PART_SPACING - 2 * PART_INNER_MARGIN;
        let mut rect = Rect::new(0,
                                 TRAY_OUTER_MARGIN,
                                 tray_width,
                                 window_size.height - 2 * TRAY_OUTER_MARGIN);

        let mut num_parts: usize = 0;
        let mut categories = Vec::<(&str, Vec<ChipType>)>::new();
        for &(name, ctypes) in CHIP_CATEGORIES.iter() {
            let allowed_ctypes: Vec<ChipType> = ctypes
                .iter()
                .cloned()
                .filter(|ctype| ctype.is_allowed_in(current_puzzle))
                .collect();
            if !allowed_ctypes.is_empty() {
                num_parts += allowed_ctypes.len();
                categories.push((name, allowed_ctypes));
            }
        }

        let mut category_labels =
            Vec::<CategoryLabel>::with_capacity(categories.len());
        let mut parts = Vec::<Part>::with_capacity(num_parts);
        let mut top = rect.y + TRAY_INNER_MARGIN;
        for (name, ctypes) in categories {
            category_labels.push(CategoryLabel::new(top, name));
            top += CATEGORY_LABEL_HEIGHT - PART_INNER_MARGIN;
            let mut col = 0;
            for ctype in ctypes {
                let left = TRAY_INNER_MARGIN - PART_INNER_MARGIN +
                    col * (PART_WIDTH + PART_SPACING);
                parts.push(Part::new(left, top, ctype));
                col += 1;
                if col >= num_columns {
                    col = 0;
                    top += PART_HEIGHT + PART_SPACING;
                }
            }
            if col > 0 {
                top += PART_HEIGHT + PART_SPACING;
            }
        }

        let mut scrollbar =
            Scrollbar::new(Rect::new(rect.right() - TRAY_INNER_MARGIN +
                                         SCROLLBAR_MARGIN,
                                     rect.y + TRAY_INNER_MARGIN,
                                     SCROLLBAR_WIDTH,
                                     rect.height - 2 * TRAY_INNER_MARGIN));
        scrollbar.set_total_height(top + TRAY_INNER_MARGIN - rect.y);
        if scrollbar.is_visible() {
            rect.width += 2 * SCROLLBAR_MARGIN + SCROLLBAR_WIDTH -
                TRAY_INNER_MARGIN;
        }

        PartsTray {
            rect,
            category_labels,
            parts,
            scrollbar,
            slide: TraySlide::new(rect.width),
        }
    }

    fn slid_rect(&self) -> Rect<i32> {
        self.rect - vec2(self.slide.distance(), 0)
    }

    pub fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let matrix = matrix *
            Matrix4::trans2(-self.slide.distance() as f32, 0.0);
        {
            let stencil = Stencil::new();
            self.draw_box(resources, &matrix);
            stencil.enable_clipping();
            self.draw_parts(resources, &matrix);
        }
        self.scrollbar.draw(resources, &matrix);
    }

    fn draw_box(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let ui = resources.shaders().ui();
        let rect = self.rect.as_f32();
        let tab_rect =
            UiShader::tray_tab_rect(rect, TRAY_TAB_HEIGHT, TRAY_FLIP_HORZ);

        ui.draw_tray(matrix,
                     &rect,
                     TRAY_TAB_HEIGHT,
                     TRAY_FLIP_HORZ,
                     &Color4::ORANGE2,
                     &Color4::CYAN2,
                     &Color4::PURPLE0.with_alpha(0.8));

        let tab_matrix = matrix *
            Matrix4::trans2(tab_rect.x + 0.5 * tab_rect.width,
                            tab_rect.y + 0.5 * tab_rect.height) *
            Matrix4::from_angle_z(Deg(-90.0));
        let font = resources.fonts().roman();
        font.draw(&tab_matrix,
                  TRAY_TAB_FONT_SIZE,
                  Align::MidCenter,
                  (0.0, -2.0),
                  TRAY_TAB_TEXT);
    }

    fn draw_parts(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let scroll = self.scrollbar.scroll_top() as f32;
        let matrix = matrix * Matrix4::trans2(0.0, -scroll);
        for label in self.category_labels.iter() {
            label.draw(resources, &matrix);
        }
        for part in self.parts.iter() {
            part.draw(resources, &matrix);
        }
    }

    pub fn on_event(&mut self, event: &Event, ui: &mut Ui)
                    -> (Option<PartsAction>, bool) {
        let rel_event =
            event.relative_to(Point2::new(-self.slide.distance(), 0));
        self.scrollbar.on_event(&rel_event);

        match event {
            Event::ClockTick(tick) => {
                self.slide.on_tick(tick);
            }
            Event::MouseDown(mouse) if mouse.left => {
                let rel_mouse_pt = mouse.pt + vec2(self.slide.distance(), 0);
                let tab_rect = UiShader::tray_tab_rect(self.rect.as_f32(),
                                                       TRAY_TAB_HEIGHT,
                                                       TRAY_FLIP_HORZ);
                if tab_rect.contains_point(rel_mouse_pt.as_f32()) {
                    self.slide.toggle();
                    return (None, true);
                } else if self.rect.contains_point(rel_mouse_pt) {
                    let rel_scrolled_pt = rel_mouse_pt +
                        vec2(0, self.scrollbar.scroll_top());
                    for part in self.parts.iter() {
                        if part.rect.contains_point(rel_scrolled_pt) {
                            let action = PartsAction::Grab(part.ctype,
                                                           mouse.pt);
                            return (Some(action), true);
                        }
                    }
                    return (None, true);
                }
            }
            Event::MouseMove(mouse) => {
                if let Some(cursor) = self.cursor_for_mouse_pt(mouse.pt) {
                    ui.cursor().request(cursor);
                }
            }
            Event::MouseUp(mouse) => {
                if let Some(cursor) = self.cursor_for_mouse_pt(mouse.pt) {
                    ui.cursor().request(cursor);
                }
                if mouse.left && self.slid_rect().contains_point(mouse.pt) {
                    return (Some(PartsAction::Drop), false);
                }
            }
            Event::Multitouch(touch)
                if self.slid_rect().contains_point(touch.pt) => {
                return (None, true);
            }
            Event::Scroll(scroll)
                if self.slid_rect().contains_point(scroll.pt) => {
                self.scrollbar.scroll_by(scroll.delta.y);
                return (None, true);
            }
            _ => {}
        }
        return (None, false);
    }

    fn cursor_for_mouse_pt(&self, mouse_pt: Point2<i32>) -> Option<Cursor> {
        let rel_mouse_pt = mouse_pt + vec2(self.slide.distance(), 0);
        let tab_rect = UiShader::tray_tab_rect(self.rect.as_f32(),
                                               TRAY_TAB_HEIGHT,
                                               TRAY_FLIP_HORZ);
        if tab_rect.contains_point(rel_mouse_pt.as_f32()) {
            return Some(Cursor::default());
        } else if self.rect.contains_point(rel_mouse_pt) {
            let rel_scrolled_pt = rel_mouse_pt +
                vec2(0, self.scrollbar.scroll_top());
            for part in self.parts.iter() {
                if part.rect.contains_point(rel_scrolled_pt) {
                    return Some(Cursor::HandOpen);
                }
            }
            return Some(Cursor::default());
        }
        return None;
    }
}

//===========================================================================//

struct CategoryLabel {
    top: i32,
    text: &'static str,
}

impl CategoryLabel {
    fn new(top: i32, text: &'static str) -> CategoryLabel {
        CategoryLabel { top, text }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        resources.fonts().roman().draw(matrix,
                                       CATEGORY_LABEL_FONT_SIZE,
                                       Align::TopLeft,
                                       (TRAY_INNER_MARGIN as f32,
                                        self.top as f32),
                                       self.text);
    }
}

//===========================================================================//

struct Part {
    rect: Rect<i32>,
    ctype: ChipType,
}

impl Part {
    fn new(left: i32, top: i32, ctype: ChipType) -> Part {
        let rect = Rect::new(left, top, PART_WIDTH, PART_HEIGHT);
        Part { rect, ctype }
    }

    fn draw(&self, resources: &Resources, matrix: &Matrix4<f32>) {
        let chip_size = self.ctype.size();
        let chip_dim = chip_size.width.max(chip_size.height) as f32;
        let rect = self.rect.expand(-PART_INNER_MARGIN).as_f32();
        let matrix = matrix * Matrix4::trans2(rect.x, rect.y) *
            Matrix4::from_scale(rect.width / chip_dim);
        ChipModel::draw_chip(resources,
                             &matrix,
                             self.ctype,
                             Orientation::default(),
                             None);
    }
}

//===========================================================================//
