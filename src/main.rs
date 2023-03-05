mod adsr;
mod lerp_note;
mod low_pass_filter;
mod play_source;
mod sound_commands;
mod wave_table;

use adsr::Adsr;
use play_source::{drop_mixer, init_mixer, play, reset_mixer};
use sound_commands::{set_adsr, set_frequency, set_lerp_time, set_note_state, set_volume};
use wave_table::{WaveForm, WaveTable};

fn play_test(f: f32) {}

fn guile() {
    // init_scm();
    // register_void_function!(b"create\0", create_test);
    // register_args_function!(b"read\0", 1, read_test);
    // run_scm(0, [].as_mut_ptr());
}

fn preload() {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let (filtrado, ctrl) = Adsr::new(WaveTable::new(44100, WaveForm::Saw));
    set_frequency(&ctrl, 0.0);
    set_volume(&ctrl, 0.0);
    set_adsr(&ctrl, 0.075, 0.025, 0.55, 0.2);

    play(filtrado);

    set_volume(&ctrl, 0.1);

    for _ in 0..2 * 5 {
        set_frequency(&ctrl, 220.0);

        set_note_state(&ctrl, true);
        std::thread::sleep(std::time::Duration::from_millis(150));
        set_note_state(&ctrl, false);
        std::thread::sleep(std::time::Duration::from_millis(150));

        set_frequency(&ctrl, 440.0);

        set_note_state(&ctrl, true);
        std::thread::sleep(std::time::Duration::from_millis(150));
        set_note_state(&ctrl, false);
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}

fn main() {
    init_mixer();
    preload();
    std::thread::sleep(std::time::Duration::from_millis(500));
    guile();
    drop_mixer();
}
