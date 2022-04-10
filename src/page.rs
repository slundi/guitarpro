use fraction::ToPrimitive;

use crate::{gp::*, io::*};

///A padding construct
#[derive(Debug,Clone)]
pub struct Padding {
    pub right: u16,
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
}

/// A point construct using integer coordinates
#[derive(Debug,Clone)]
pub struct Point { pub x: u16, pub y: u16, }

// An enumeration of the elements which can be shown in the header and footer of a rendered song sheet.
// All values can be combined using bit-operators as they are flags
pub const HEADER_FOOTER_NONE: u16 = 0x000;
pub const HEADER_FOOTER_TITLE: u16 = 0x001;
pub const HEADER_FOOTER_SUBTITLE: u16 = 0x002;
pub const HEADER_FOOTER_ARTIST: u16 = 0x004;
pub const HEADER_FOOTER_ALBUM: u16 = 0x008;
pub const HEADER_FOOTER_WORDS: u16 = 0x010;
pub const HEADER_FOOTER_MUSIC: u16 = 0x020;
pub const HEADER_FOOTER_WORD_AND_MUSIC: u16 = 0x040;
pub const HEADER_FOOTER_COPYRIGHT: u16 = 0x080;
pub const HEADER_FOOTER_PAGE_NUMBER: u16 = 0x100;
pub const HEADER_FOOTER_ALL: u16 = 0x1ff;

/// The page setup describes how the document is rendered.
/// 
/// Page setup contains page size, margins, paddings, and how the title elements are rendered.
/// 
/// Following template vars are available for defining the page texts:
/// * ``%title%``: will be replaced with Song.title
/// - ``%subtitle%``: will be replaced with Song.subtitle
/// - ``%artist%``: will be replaced with Song.artist
/// - ``%album%``: will be replaced with Song.album
/// - ``%words%``: will be replaced with Song.words
/// - ``%music%``: will be replaced with Song.music
/// - ``%WORDSANDMUSIC%``: will be replaced with the according word and music values
/// - ``%copyright%``: will be replaced with Song.copyright
/// - ``%N%``: will be replaced with the current page number (if supported by layout)
/// - ``%P%``: will be replaced with the number of pages (if supported by layout)
#[derive(Debug,Clone)]
pub struct PageSetup {
    pub page_size: Point,
    pub page_margin: Padding,
    pub score_size_proportion: f32,
    pub header_and_footer: u16,
    pub title: String,
    pub subtitle: String, //Guitar Pro
	pub artist: String,
	pub album: String,
    pub words: String, //GP
	pub music: String,
	pub word_and_music: String,
	pub copyright: String,
    pub page_number: String,
}
impl Default for PageSetup {fn default() -> Self { PageSetup { page_size:Point{x:210,y:297}, page_margin:Padding{right:10,top:15,left:10,bottom:10}, score_size_proportion:1.0, header_and_footer:HEADER_FOOTER_ALL,
    title:String::from("%title%"), subtitle:String::from("%subtitle%"), artist:String::from("%artist%"), album:String::from("%album%"),
    words:String::from("Words by %words%"), music:String::from("Music by %music%"), word_and_music:String::from("Words & Music by %WORDSMUSIC%"),
    copyright:String::from("Copyright %copyright%\nAll Rights Reserved - International Copyright Secured"),
    page_number:String::from("Page %N%/%P%"),
}}}

impl Song {
    /// Read page setup. Page setup is read as follows:
    /// - Page size: 2 :ref:`Ints <int>`. Width and height of the page.
    /// - Page padding: 4 :ref:`Ints <int>`. Left, right, top, bottom padding of the page.
    /// - Score size proportion: :ref:`int`.
    /// - Header and footer elements: :ref:`short`. See :class:`guitarpro.models.HeaderFooterElements` for value mapping.
    /// - List of placeholders:
    ///   * title
    ///   * subtitle
    ///   * artist
    ///   * album
    ///   * words
    ///   * music
    ///   * wordsAndMusic
    ///   * copyright1, e.g. *"Copyright %copyright%"*
    ///   * copyright2, e.g. *"All Rights Reserved - International Copyright Secured"*
    ///   * pageNumber
    pub fn read_page_setup(&mut self, data: &[u8], seek: &mut usize) {
        let mut s = PageSetup::default();
        s.page_size.x = read_int(data, seek).to_u16().unwrap();
        s.page_size.y = read_int(data, seek).to_u16().unwrap();
        s.page_margin.left   = read_int(data, seek).to_u16().unwrap();
        s.page_margin.right  = read_int(data, seek).to_u16().unwrap();
        s.page_margin.top    = read_int(data, seek).to_u16().unwrap();
        s.page_margin.bottom = read_int(data, seek).to_u16().unwrap();
        s.score_size_proportion = read_int(data, seek).to_f32().unwrap() / 100.0;
        s.header_and_footer = read_short(data, seek).to_u16().unwrap();
        s.title = read_int_size_string(data, seek);
        s.subtitle = read_int_size_string(data, seek);
        s.artist = read_int_size_string(data, seek);
        s.album = read_int_size_string(data, seek);
        s.words = read_int_size_string(data, seek);
        s.music = read_int_size_string(data, seek);
        s.word_and_music = read_int_size_string(data, seek);
        let mut c = read_int_size_string(data, seek);
        c.push('\n');
        c.push_str(&read_int_size_string(data, seek));
        s.copyright = c;
        s.page_number = read_int_size_string(data, seek);
    }
}
