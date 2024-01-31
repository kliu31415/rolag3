#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageColor {
    Red,
    Green,
    Blue,
    Silver,
    NotSet,
}

impl DamageColor {
    pub fn get_damage_mult(dealer: DamageColor, receiver: DamageColor) -> f64 {
        assert!(dealer != DamageColor::NotSet);
        assert!(receiver != DamageColor::NotSet);

        if dealer == DamageColor::Silver || receiver == DamageColor::Silver {
            return 1.0;
        }
        if dealer != receiver {
            return 1.0;
        }
        0.0
    }

    pub fn _to_rgb_idx(&self) -> usize {
        match self {
            DamageColor::Red => 0,
            DamageColor::Green => 1,
            DamageColor::Blue => 2,
            _ => panic!("can't convert DamageColor({:?}) to rgb index", self),
        }
    }
}