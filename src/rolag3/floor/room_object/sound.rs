use crate::sfx::sound_system::{SoundPlayingRef, SoundDataRef};

pub struct RoomObjPlaySoundArgs {
    pub sound_data: SoundDataRef,
    pub location: Option<(f64, f64)>,
    pub owned: bool,
}

pub struct RoomObjPlaySoundArgsBuilderReq {
    pub sound_data: SoundDataRef,
}

pub struct RoomObjPlaySoundArgsBuilder {
    req: RoomObjPlaySoundArgsBuilderReq,

    location: Option<(f64, f64)>,

    // if owned is false, then the played sound is unaffected by the parent RoomObject's modifiers, e.g. if the parent
    // RoomObjects wants the playback speed of all sounds it owned to be halved, unowned sounds won't be affected.
    owned: bool,
}

impl RoomObjPlaySoundArgsBuilder {
    pub fn new(req: RoomObjPlaySoundArgsBuilderReq) -> Self {
        Self {
            req,
            location: None,
            owned: true,
        }
    }

    pub fn build(self) -> RoomObjPlaySoundArgs {
        RoomObjPlaySoundArgs {
            sound_data: self.req.sound_data,
            location: self.location,
            owned: self.owned,
        }
    }
}

pub struct RoomObjSoundRef {
    _id: u128,
}

impl RoomObjSoundRef {
    pub fn new(id: u128) -> Self {
        Self {
            _id: id
        }
    }
}

pub struct RoomObjSound {
    pub id: u128,
    pub location: Option<(f64, f64)>,
    pub sound_ref: SoundPlayingRef,
}