#![allow(dead_code)]

use core::time::Duration;
use rodio::Source;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::sound_commands::SoundCommands;

fn gen_sine_wavetable(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| (2.0 * std::f32::consts::PI * n as f32 / size as f32).sin())
        .collect()
}

fn gen_triangle_wavetable(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| {
            let ft = n as f32 / size as f32;
            4.0 * (ft - (ft + 0.5).floor()).abs() - 1.0
        })
        .collect()
}

fn gen_square_wavetable(size: usize) -> Vec<f32> {
    (0..size)
        .map(|n| {
            (2.0 * std::f32::consts::PI * n as f32 / size as f32)
                .sin()
                .signum()
        })
        .collect()
}

fn gen_saw_wavetable(size: usize) -> Vec<f32> {
    let mut half1: Vec<f32> = (0..size / 2)
        .map(|n| n as f32 * (2.0 / size as f32))
        .collect();
    let half2: Vec<f32> = half1.clone().iter().rev().map(|el| el * -1.0).collect();
    half1.extend(half2);
    half1
}

pub enum WaveForm {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct WaveTable {
    sample_rate: u32,
    wave_table: Vec<f32>,
    volume: f32,
    index: f32,
    index_increment: f32,
    recv: Receiver<SoundCommands>,
}

impl WaveTable {
    pub fn new(sample_rate: u32, wave_form: WaveForm) -> (WaveTable, Sender<SoundCommands>) {
        let wave_table_size = 64;

        let (send, recv) = channel();

        let wave_table = match wave_form {
            WaveForm::Sine => gen_sine_wavetable(wave_table_size),
            WaveForm::Square => gen_square_wavetable(wave_table_size),
            WaveForm::Saw => gen_saw_wavetable(wave_table_size),
            WaveForm::Triangle => gen_triangle_wavetable(wave_table_size),
        };

        let ret = WaveTable {
            sample_rate,
            wave_table,
            volume: 0.1,
            index: 0.0,
            index_increment: 0.0,
            recv,
        };

        return (ret, send);
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn get_sample(&mut self) -> f32 {
        match self.recv.try_recv() {
            Ok(SoundCommands::Frq(frq)) => self.set_frequency(frq),
            Ok(SoundCommands::Vol(vol)) => self.set_volume(vol),
            _ => {}
        }

        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        return sample * self.volume;
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return truncated_index_weight * self.wave_table[truncated_index]
            + next_index_weight * self.wave_table[next_index];
    }
}

impl Source for WaveTable {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}

impl Iterator for WaveTable {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}
