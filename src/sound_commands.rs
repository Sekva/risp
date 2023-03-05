#![allow(dead_code)]
use rodio::Source;
use std::sync::mpsc::Sender;

pub trait SoundMaker: Source + Iterator<Item = f32> + Send + 'static {}
impl<T> SoundMaker for T where T: Source + Iterator<Item = f32> + Send + 'static {}

pub enum SoundCommands {
    Vol(f32),
    Frq(f32),
    Res(f32),
    LerpTime(f32),
    Adsr(f32, f32, f32, f32),
    NotePlaying(bool),
}

pub fn set_volume(control: &Sender<SoundCommands>, val: f32) {
    let _ = control.send(SoundCommands::Vol(val));
}

pub fn set_frequency(control: &Sender<SoundCommands>, val: f32) {
    let _ = control.send(SoundCommands::Frq(val));
}

pub fn set_ressonance(control: &Sender<SoundCommands>, val: f32) {
    let _ = control.send(SoundCommands::Res(val));
}

pub fn set_lerp_time(control: &Sender<SoundCommands>, val: f32) {
    let _ = control.send(SoundCommands::LerpTime(val));
}

pub fn set_adsr(control: &Sender<SoundCommands>, a: f32, d: f32, s: f32, r: f32) {
    let _ = control.send(SoundCommands::Adsr(a, d, s, r));
}

pub fn set_note_state(control: &Sender<SoundCommands>, val: bool) {
    let _ = control.send(SoundCommands::NotePlaying(val));
}
