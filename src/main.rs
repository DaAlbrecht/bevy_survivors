use bevy::prelude::*;
use bevy_seedling::prelude::{MainBus, Volume, VolumeNode};

fn main() -> AppExit {
    let mute = std::env::args().any(|arg| arg == "--mute");

    App::new()
        .add_plugins(bevy_survivors::plugin)
        .add_systems(PostStartup, (mute_global_audio).run_if(move || mute))
        .run()
}

fn mute_global_audio(mut master: Single<&mut VolumeNode, With<MainBus>>) {
    master.volume = Volume::Linear(0.);
}
