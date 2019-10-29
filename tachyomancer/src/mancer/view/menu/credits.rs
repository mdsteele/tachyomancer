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

use super::super::paragraph::Paragraph;
use crate::mancer::save::Prefs;

//===========================================================================//

const PARAGRAPH_FONT_SIZE: f32 = 20.0;
const PARAGRAPH_LINE_HEIGHT: f32 = 22.0;

const FIRST_PARAGRAPH_FORMAT: &str =
    "$=$O$*TACHYOMANCER$*$D$<\n\
     $=\u{a9}2018 Matthew D. Steele <mdsteele@alum.mit.edu>$<\n\n\
     $CGame website:$D$>https://mdsteele.games/tachyomancer/$<\n\
     $CSource code:$D$>https://github.com/mdsteele/tachyomancer/$<\n\n\
     $/Tachyomancer$/ comes with ABSOLUTELY NO WARRANTY.\n\
     $/Tachyomancer$/ is free software: you can redistribute it and/or modify \
     it under the terms of the GNU General Public License as published by the \
     Free Software Foundation, either version 3 of the License, or (at your \
     option) any later version.\n\n\
     $=Thanks for playing!";

const GAME_PARAGRAPH_FORMAT: &str =
    "    $O$*GAME$*$D\n\
     $CDesign/Programming:$D Matthew Steele\n     \
     $CAlpha Testing:$D TODO\n      \
     $CBeta Testing:$D TODO";

//===========================================================================//

struct ResourceInfo {
    name: &'static str,
    artist: &'static str,
    license: &'static str,
    year: i32,
    url: &'static str,
}

// Generated code:
// const FONT_RESOURCE_INFO: &[ResourceInfo] = ...
include!(concat!(env!("OUT_DIR"), "/rsrc_info/font.rs"));

// Generated code:
// const MUSIC_RESOURCE_INFO: &[ResourceInfo] = ...
include!(concat!(env!("OUT_DIR"), "/rsrc_info/music.rs"));

// Generated code:
// const SOUND_RESOURCE_INFO: &[ResourceInfo] = ...
include!(concat!(env!("OUT_DIR"), "/rsrc_info/sound.rs"));

// Generated code:
// const TEXTURE_RESOURCE_INFO: &[ResourceInfo] = ...
include!(concat!(env!("OUT_DIR"), "/rsrc_info/texture.rs"));

//===========================================================================//

pub fn credits_paragraphs(max_width: f32, prefs: &Prefs) -> Vec<Paragraph> {
    vec![
        Paragraph::compile(
            PARAGRAPH_FONT_SIZE,
            PARAGRAPH_LINE_HEIGHT,
            max_width,
            prefs,
            FIRST_PARAGRAPH_FORMAT,
        ),
        Paragraph::compile(
            PARAGRAPH_FONT_SIZE,
            PARAGRAPH_LINE_HEIGHT,
            max_width,
            prefs,
            GAME_PARAGRAPH_FORMAT,
        ),
        rsrc_paragraph("FONTS", FONT_RESOURCE_INFO, max_width, prefs),
        rsrc_paragraph("MUSIC", MUSIC_RESOURCE_INFO, max_width, prefs),
        rsrc_paragraph("SOUNDS", SOUND_RESOURCE_INFO, max_width, prefs),
        rsrc_paragraph("TEXTURES", TEXTURE_RESOURCE_INFO, max_width, prefs),
    ]
}

fn rsrc_paragraph(
    title: &str,
    infos: &[ResourceInfo],
    max_width: f32,
    prefs: &Prefs,
) -> Paragraph {
    let mut formats = Vec::<String>::new();
    formats.push(format!("    $O$*{}$*$D", title));
    for info in infos.iter() {
        formats.push(format!(
            "$C{}$D $>\u{a9}{} {} ({})$<\n$<{}",
            Paragraph::escape(info.name),
            info.year,
            Paragraph::escape(info.artist),
            Paragraph::escape(info.license),
            Paragraph::escape(info.url)
        ));
    }
    Paragraph::compile(
        PARAGRAPH_FONT_SIZE,
        PARAGRAPH_LINE_HEIGHT,
        max_width,
        prefs,
        &formats.join("\n"),
    )
}

//===========================================================================//
