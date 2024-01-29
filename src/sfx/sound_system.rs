use std::{error::Error, collections::BTreeMap};

use kira::{manager::{AudioManagerSettings, AudioManager}, sound::{static_sound::{StaticSoundSettings, StaticSoundData, StaticSoundHandle}, PlaybackState}, tween::Tween};

pub trait SoundSystem {
    fn load_sound_data_file_into_db(&mut self, path: &str) -> Result<SoundDataRef, Box<dyn Error>>;
    fn play_sound(&mut self, args: PlaySoundArgs) -> Result<SoundPlayingRef, Box<dyn Error>>;
    fn is_done_playing(&mut self, spr: &SoundPlayingRef) -> bool;
    // returns Ok(false) if the sound is done playing. The caller should then delete the SoundPlayingRef, because the
    // played sound associated with the SoundPlayingRef is deleted from the SoundSystem's internal data.
    fn modify_sound_playing_if_not_done(&mut self, spr: &SoundPlayingRef, modify: ModifySoundPlaying) -> Result<bool, Box<dyn Error>>;
}

pub struct PlaySoundArgs {
    pub sdr: SoundDataRef,
    pub panning: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct SoundDataRef {
    id: usize,
}

pub struct SoundPlayingRef {
    id: usize,
}

pub enum ModifySoundPlaying {
    _PlaybackRate {rate: f64},
}

struct KiraSoundSystem {
    audio_manager: AudioManager,

    static_sound_db_id_counter: usize,
    static_sound_db: BTreeMap<usize, StaticSoundData>,

    sound_playing_id_counter: usize,
    sound_playing: BTreeMap<usize, StaticSoundHandle>,
}

const IMMEDIATE_TWEEN: Tween = Tween {
    start_time: kira::StartTime::Immediate,
    duration: std::time::Duration::from_nanos(0),
    easing: kira::tween::Easing::Linear,
};

impl SoundSystem for KiraSoundSystem {
    fn load_sound_data_file_into_db(&mut self, path: &str) -> Result<SoundDataRef, Box<dyn Error>> {
        let sound_data = StaticSoundData::from_file(path, StaticSoundSettings::new())?;
        self.static_sound_db_id_counter += 1;
        self.static_sound_db.insert(self.static_sound_db_id_counter, sound_data);
        Ok(SoundDataRef {
            id: self.static_sound_db_id_counter,
        })
    }

    fn play_sound(&mut self, args: PlaySoundArgs) -> Result<SoundPlayingRef, Box<dyn Error>> {
        let sound_data = self.static_sound_db.get(&args.sdr.id).unwrap().clone();
        let mut sound = self.audio_manager.play(sound_data)?;
        sound.set_panning(args.panning, IMMEDIATE_TWEEN)?;
        self.sound_playing_id_counter += 1;
        self.sound_playing.insert(self.sound_playing_id_counter, sound);
        Ok(SoundPlayingRef {
            id: self.sound_playing_id_counter,
        })
    }

    fn is_done_playing(&mut self, spr: &SoundPlayingRef) -> bool {
        let sh = self.sound_playing.get_mut(&spr.id);
        let Some(sh) = sh else {
            panic!("unable to get StaticSoundHandle with id={}. It was likely deleted earlier.", &spr.id);
        };

        if sh.state() == PlaybackState::Stopped {
            self.sound_playing.remove(&spr.id);
            return false;
        }
        true
    }

    fn modify_sound_playing_if_not_done(&mut self, spr: &SoundPlayingRef, modify: ModifySoundPlaying) -> Result<bool, Box<dyn Error>> {
        let sh = self.sound_playing.get_mut(&spr.id);
        let Some(sh) = sh else {
            panic!("unable to get StaticSoundHandle with id={}. It was likely deleted earlier.", &spr.id);
        };

        if sh.state() == PlaybackState::Stopped {
            self.sound_playing.remove(&spr.id);
            return Ok(false);
        }

        match modify {
            ModifySoundPlaying::_PlaybackRate { rate } => {
                sh.set_playback_rate(rate, IMMEDIATE_TWEEN)?;
            }
        }

        Ok(true)
    }
}

pub fn new_sound_system() -> Result<Box<dyn SoundSystem>, Box<dyn Error>> {
    let audio_manager = AudioManager::new(AudioManagerSettings::default())?;
    Ok(Box::new(KiraSoundSystem {
        audio_manager,

        static_sound_db_id_counter: 0,
        static_sound_db: BTreeMap::new(),

        sound_playing_id_counter: 0,
        sound_playing: BTreeMap::new(),
    }))
}