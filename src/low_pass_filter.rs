use std::sync::mpsc::{channel, Receiver, Sender};

use rodio::Source;

use crate::sound_commands::{SoundCommands, SoundMaker};

pub struct LowPassFilter<T>
where
    T: SoundMaker,
{
    cutoff_frequency: f32,
    rc: f32,
    dt: f32,
    alpha: f32,
    source: T,
    sample_rate: u32,
    last_sample: Option<f32>,
    recv: Receiver<SoundCommands>,
    other_control: Sender<SoundCommands>,
}

impl<T: SoundMaker> LowPassFilter<T> {
    pub fn new(
        (source, other_control): (T, Sender<SoundCommands>),
    ) -> (Self, Sender<SoundCommands>) {
        let cutoff_frequency = 110.0;
        let sample_rate = source.sample_rate();
        let dt = 1.0 / sample_rate as f32;
        let rc = 1.0 / (2.0 * cutoff_frequency * core::f32::consts::PI);
        let alpha = dt / (rc + dt);

        let last_sample = None;

        let (send, recv) = channel();

        let ret = Self {
            cutoff_frequency,
            rc,
            dt,
            alpha,
            source,
            sample_rate,
            last_sample,
            recv,
            other_control,
        };

        (ret, send)
    }

    fn get_sample(&mut self) -> f32 {
        match self.recv.try_recv() {
            Ok(SoundCommands::Res(v)) => self.set_cutoff_frequency(v),
            Ok(v) => {
                let _ = self.other_control.send(v);
            }

            _ => {}
        }

        let next_sample = self.source.next().unwrap();

        match self.last_sample {
            Some(prev_sample) => {
                let out = prev_sample + self.alpha * (next_sample - prev_sample);
                self.last_sample.replace(out);
                return out;
            }
            None => {
                let out = self.alpha * next_sample;
                self.last_sample.replace(out);
                return out;
            }
        }
    }

    fn set_cutoff_frequency(&mut self, freq: f32) {
        self.cutoff_frequency = freq;
        self.rc = 1.0 / (2.0 * self.cutoff_frequency * core::f32::consts::PI);
        self.alpha = self.dt / (self.rc + self.dt);
    }
}

impl<T: SoundMaker> Iterator for LowPassFilter<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}

impl<T: SoundMaker> Source for LowPassFilter<T> {
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
