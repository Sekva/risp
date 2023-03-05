use std::sync::mpsc::{channel, Receiver, Sender};

use rodio::Source;

use crate::sound_commands::{set_frequency, SoundCommands, SoundMaker};

pub struct Lerp<T>
where
    T: SoundMaker,
{
    source: T,
    sample_rate: u32,

    curr_freq: f32,
    targ_freq: f32,
    freq_step: f32,
    time: f32,

    recv: Receiver<SoundCommands>,
    other_control: Sender<SoundCommands>,
}

impl<T: SoundMaker> Lerp<T> {
    pub fn new(
        (source, other_control): (T, Sender<SoundCommands>),
    ) -> (Self, Sender<SoundCommands>) {
        let (send, recv) = channel();

        let freq_step = 0.0;
        let curr_freq = 0.0;
        let targ_freq = 0.0;

        let sample_rate = source.sample_rate();

        let time = 0.5;

        let this = Lerp {
            source,
            sample_rate,
            curr_freq,
            targ_freq,
            freq_step,
            recv,
            other_control,
            time,
        };

        (this, send)
    }

    fn update_freq(&mut self) {
        self.curr_freq += self.freq_step;
        set_frequency(&self.other_control, self.curr_freq);

        let diff = self.targ_freq - self.curr_freq;

        if diff.abs() < 0.1 {
            self.curr_freq = self.targ_freq;
            self.freq_step = 0.0;
        }
    }

    fn set_lerp(&mut self, to_freq: f32) {
        self.targ_freq = to_freq;
        let diff = self.targ_freq - self.curr_freq;

        if diff.abs() < 0.1 {
            self.curr_freq = self.targ_freq;
            self.freq_step = 0.0;
        }

        let samples = self.time * self.sample_rate as f32;
        let step = diff / samples;

        self.freq_step = step;
    }

    fn get_sample(&mut self) -> f32 {
        match self.recv.try_recv() {
            Ok(SoundCommands::Frq(f)) => self.set_lerp(f),
            Ok(SoundCommands::LerpTime(t)) => {
                self.time = t;
                self.set_lerp(self.targ_freq);
            }
            Ok(v) => {
                let _ = self.other_control.send(v);
            }

            _ => {}
        }

        self.update_freq();
        self.source.next().unwrap()
    }
}

impl<T: SoundMaker> Iterator for Lerp<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_sample())
    }
}

impl<T: SoundMaker> Source for Lerp<T> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
