use rodio::{
    dynamic_mixer::{mixer, DynamicMixerController},
    OutputStream, Source,
};
use std::sync::{Arc, Once};

use crate::{
    sound_commands::{set_frequency, set_volume, SoundMaker},
    wave_table::{WaveForm, WaveTable},
};

static mut OUTPUT_STREAM: Option<OutputStream> = None;
static mut MIXER_CONTROL: Option<Box<Arc<DynamicMixerController<f32>>>> = None;
static mut INIT_SOUND: Once = Once::new();

pub fn reset_mixer() {
    drop_mixer();
    init_mixer();
}

pub fn init_mixer() {
    configure_audio_output();
    let (osc, ctrl) = WaveTable::new(44100, WaveForm::Square);
    set_frequency(&ctrl, 0.0);
    set_volume(&ctrl, 0.0);
    play(osc);
}

pub fn configure_audio_output() {
    unsafe {
        INIT_SOUND.call_once(|| {
            let (controller, out) = mixer::<f32>(1, 44100);
            let (stream, stream_handle) = OutputStream::try_default().unwrap();
            let _ = stream_handle.play_raw(out.convert_samples());
            OUTPUT_STREAM.replace(stream);
            MIXER_CONTROL.replace(Box::new(controller));
        });
    }
}

pub fn drop_mixer() {
    unsafe {
        OUTPUT_STREAM = None;
        MIXER_CONTROL = None;
        INIT_SOUND = Once::new();
    }
}

pub fn play<T>(src: T)
where
    T: SoundMaker,
{
    unsafe {
        match MIXER_CONTROL.as_ref() {
            Some(mixer) => mixer.add(src.fade_in(std::time::Duration::from_millis(1))),
            None => todo!("Mensagem mixer não iniciado"),
        }
    }
}
