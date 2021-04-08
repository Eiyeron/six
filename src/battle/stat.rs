use rand::Rng;

#[derive(Clone, Copy)]
pub struct Stat {
    pub base: u16,
    pub modifier: i16,
}

impl Stat {
    pub fn new(base: u16) -> Stat {
        Stat { base, modifier: 0 }
    }
    pub fn multiplied(&self) -> u16 {
        let mult: f32 = get_stat_multiplier(self.modifier) * f32::from(self.base);
        mult as u16
    }

    // Returns the stat difference after buff
    pub fn buff(&mut self, levels: i16) -> i16 {
        let current_value = self.multiplied() as i16;
        self.modifier = i16::clamp(self.modifier + levels, -3, 3);
        let new_value = self.multiplied() as i16;

        new_value - current_value
    }
}

//Copying it for action exectuion
#[derive(Clone)]
pub struct ActorStats {
    pub offense: Stat,
    pub defense: Stat,
    pub speed: Stat,
    pub iq: Stat,
    // TODO guts
}

// Logic code decorrelated from structs
fn get_stat_multiplier(modifier: i16) -> f32 {
    match modifier {
        -3 => 0.125,
        -2 => 0.25,
        -1 => 0.5,
        0 => 1.,
        1 => 1.5,
        2 => 1.75,
        3 => 2.,
        _ => panic!("Invalid modifier"),
    }
}

pub fn damage(offense: u16, attack_level: u16, defense: u16) -> u16 {
    let mut rng = rand::thread_rng();
    let random_multiplier = rng.gen_range(0.75..1.25);
    let base = attack_level * offense - defense;
    ((base as f32) * random_multiplier) as u16
}
