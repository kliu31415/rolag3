#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageColor {
    Red,
    Green,
    Blue,
    NotSet,
    Silver,
}

impl DamageColor {
    pub fn get_damage_mult(dealer: DamageColor, receiver: DamageColor) -> f64 {
        assert!(dealer != DamageColor::NotSet);
        assert!(receiver != DamageColor::NotSet);
        assert!(receiver != DamageColor::Silver);

        if dealer == DamageColor::Silver {
            return 1.0;
        }
        if dealer != receiver {
            return 1.0;
        }
        0.0
    }
}