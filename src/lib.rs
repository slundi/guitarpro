#[macro_use]
extern crate lazy_static;

#[path = "song.rs"] mod gp;
mod io;
mod effects;
mod key_signature;
mod midi;
mod mix_table;
mod chord;
mod page;
mod rse;
mod note;
mod lyric;
