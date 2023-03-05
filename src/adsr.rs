use std::sync::mpsc::{channel, Receiver, Sender};

use rodio::Source;

use crate::sound_commands::{set_frequency, SoundCommands, SoundMaker};

pub struct Adsr<T>
where
    T: SoundMaker,
{
    source: T,
    sample_rate: u32,

    atk_factor: Vec<f32>,
    dly_factor: Vec<f32>,
    sus_factor: f32,
    rls_factor: Vec<f32>,

    idx_atk: usize,
    idx_dly: usize,
    idx_rls: usize,

    is_note_on: bool,

    recv: Receiver<SoundCommands>,
    other_control: Sender<SoundCommands>,
}

impl<T: SoundMaker> Adsr<T> {
    pub fn new(
        (source, other_control): (T, Sender<SoundCommands>),
    ) -> (Self, Sender<SoundCommands>) {
        let sample_rate = source.sample_rate();

        let (send, recv) = channel();

        let atk_factor: Vec<f32> = vec![];
        let dly_factor: Vec<f32> = vec![];
        let sus_factor: f32 = 0.0;
        let rls_factor: Vec<f32> = vec![];

        let idx_atk = std::usize::MAX;
        let idx_dly = std::usize::MAX;
        let idx_rls = std::usize::MAX;

        let is_note_on = false;

        let adsr = Adsr {
            source,
            sample_rate,
            atk_factor,
            dly_factor,
            sus_factor,
            rls_factor,
            idx_atk,
            idx_dly,
            idx_rls,
            is_note_on,
            recv,
            other_control,
        };

        (adsr, send)
    }

    fn set_adsr(&mut self, a: f32, d: f32, s: f32, r: f32) {
        let samples_atk = a * self.sample_rate as f32;
        let samples_dly = d * self.sample_rate as f32;
        let samples_rls = r * self.sample_rate as f32;

        self.sus_factor = s;

        let step_atk = 1.0 / samples_atk as f32;
        let step_dly = (s - 1.0) / samples_dly as f32;
        let step_rls = -s / samples_rls as f32;

        self.atk_factor.clear();
        self.atk_factor.clear();
        self.atk_factor.clear();

        for i in 0..(samples_atk as usize) {
            self.atk_factor.push(i as f32 * step_atk);
        }

        for i in 0..(samples_dly as usize) {
            self.dly_factor.push(1.0 + i as f32 * step_dly);
        }

        for i in 0..(samples_rls as usize) {
            self.rls_factor.push(self.sus_factor + i as f32 * step_rls);
        }
    }

    fn set_note_state(&mut self, state: bool) {
        self.is_note_on = state;

        if self.is_note_on {
            self.idx_atk = 0;
            self.idx_dly = 0;
            self.idx_rls = 0;
        }
    }

    fn get_sample(&mut self) -> f32 {
        match self.recv.try_recv() {
            Ok(SoundCommands::Adsr(a, d, s, r)) => self.set_adsr(a, d, s, r),
            Ok(SoundCommands::NotePlaying(s)) => self.set_note_state(s),
            Ok(v) => {
                let _ = self.other_control.send(v);
            }

            _ => {}
        }

        let sample = self.source.next().unwrap();

        if self.is_note_on {
            if self.idx_atk < self.atk_factor.len() {
                let ret = sample * self.atk_factor[self.idx_atk];
                self.idx_atk += 1;
                return ret;
            }

            if self.idx_dly < self.dly_factor.len() {
                let ret = sample * self.dly_factor[self.idx_dly];
                self.idx_dly += 1;
                return ret;
            }

            return sample * self.sus_factor;
        }

        if self.idx_rls < self.rls_factor.len() {
            let ret = sample * self.rls_factor[self.idx_rls];
            self.idx_rls += 1;
            return ret;
        }

        return 0.0;
    }
}

impl<T: SoundMaker> Iterator for Adsr<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_sample())
    }
}

impl<T: SoundMaker> Source for Adsr<T> {
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
