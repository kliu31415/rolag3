use crate::sfx::sound_system::{SoundDataRef, SoundSystem};

pub struct SoundDb {
    pub gun_pistol_shot: [SoundDataRef; 5],
    pub explosion_small: [SoundDataRef; 4],
    pub gun_grenade_launcher_shot: [SoundDataRef; 4],
    pub sci_fi_weapon_laser_small: [SoundDataRef; 6],
}

impl SoundDb {
    pub fn new(ss: &mut dyn SoundSystem) -> SoundDb {
        SoundDb {
            gun_pistol_shot: Self::load_gm(ss, "gun_pistol_shot", 5)[..].try_into().unwrap(),
            explosion_small: Self::load_gm(ss, "explosion_small", 4)[..].try_into().unwrap(),
            gun_grenade_launcher_shot: Self::load_gm(ss, "gun_grenade_launcher_shot", 4)[..].try_into().unwrap(),
            sci_fi_weapon_laser_small: Self::load_gm(ss, "sci-fi_weapon_laser_small", 6)[..].try_into().unwrap(),
        }
    }

    fn load_gm(ss: &mut dyn SoundSystem, name: &str, n: usize) -> Box<[SoundDataRef]> {
        assert!(n < 100, "n({}) is too big", n);
        let mut result = Vec::new();
        result.reserve_exact(n);
        for i in 1..=n {
            let path = format!("audio/game_master_v1.3/{}/{}_{:0>2}.wav", name, name, i);
            match ss.load_sound_data_file_into_db(&path) {
                Ok(r) => result.push(r),
                Err(e) => panic!("unable to load sound file \"{}\", err={}", path, e),
            }
        }
        result.into()
    }
}