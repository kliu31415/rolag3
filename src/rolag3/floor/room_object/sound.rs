use crate::sfx::sound_system::{SoundPlayingRef, SoundDataRef};

pub struct RoomObjPlaySoundArgs {
    pub id: RoomObjSoundIdT,
    pub sound_data: SoundDataRef,
    pub location: Option<(f64, f64)>,
    pub volume_db_shift: f64,
    pub playback_speed: f64,
    pub owned: bool,
}

pub struct RoomObjPlaySoundArgsBuilderReq {
    pub id: RoomObjSoundIdT,
    pub sound_data: SoundDataRef,
}

pub struct RoomObjPlaySoundArgsBuilder {
    req: RoomObjPlaySoundArgsBuilderReq,

    location: Option<(f64, f64)>,
    volume_db_shift: f64,
    playback_speed: f64,

    // if owned is false, then the played sound is unaffected by the parent RoomObject's modifiers, e.g. if the parent
    // RoomObjects wants the playback speed of all sounds it owned to be halved, unowned sounds won't be affected.
    owned: bool,
}

impl RoomObjPlaySoundArgsBuilder {
    pub fn new(req: RoomObjPlaySoundArgsBuilderReq) -> Self {
        Self {
            req,
            location: None,
            volume_db_shift: 1.0,
            playback_speed: 1.0,
            owned: true,
        }
    }

    pub fn location(mut self, x: f64, y: f64) -> Self {
        self.location = Some((x, y));
        self
    }

    pub fn volume_db_shift(mut self, volume_db_shift: f64) -> Self {
        self.volume_db_shift = volume_db_shift;
        self
    }

    pub fn playback_speed(mut self, playback_speed: f64) -> Self {
        self.playback_speed = playback_speed;
        self
    }

    pub fn build(self) -> RoomObjPlaySoundArgs {
        RoomObjPlaySoundArgs {
            id: self.req.id,
            sound_data: self.req.sound_data,
            location: self.location,
            volume_db_shift: self.volume_db_shift,
            playback_speed: self.playback_speed,
            owned: self.owned,
        }
    }
}

pub type RoomObjSoundIdT = u64;

pub struct RoomObjSoundRef {
    _id: RoomObjSoundIdT,
}

impl RoomObjSoundRef {
    pub fn new(id: RoomObjSoundIdT) -> Self {
        Self {
            _id: id
        }
    }
}

pub struct RoomObjSound {
    pub id: RoomObjSoundIdT,
    pub location: Option<(f64, f64)>,
    pub sound_ref: SoundPlayingRef,
}