use crate::battle::stat::damage;
use crate::battle::stat::ActorStats;
use crate::battle::turn::SubStateTransition;
use crate::battle::turn::TurnSubState;
use crate::battle::Actor;
use crate::timer::Timer;
use tetra::Context;

pub trait Action {
    fn go(
        &mut self,
        caster_stats: &ActorStats,
        targets: &mut [Actor],
        ctx: &Context,
    ) -> SubStateTransition;
    // Cancel due to K.O.
    fn cancel(&mut self);
}

// Bash attack
// (Really don't want to mess with the music-based beat-down for now)
pub struct Bash {
    time: Timer,
    cancelled: bool,
}

impl Bash {
    pub fn new() -> Bash {
        Bash {
            time: Timer::new(1.0),
            cancelled: false,
        }
    }
}

fn physical_damage(offense: u16, attack_level: u16, targets: &mut [Actor]) {
    for target in targets.iter_mut() {
        let dmg = damage(offense, attack_level, target.stats.defense.multiplied());
        target.hp.hit(dmg);
    }
}

impl Action for Bash {
    fn go(
        &mut self,
        caster_stats: &ActorStats,
        targets: &mut [Actor],
        ctx: &Context,
    ) -> SubStateTransition {
        use SubStateTransition::*;
        if self.cancelled {
            return NextSubState(TurnSubState::NextAction);
        }
        self.time.tetra_tick(ctx);
        if !self.time.done() {
            return None;
        }

        physical_damage(caster_stats.offense.multiplied(), 1, targets);
        NextSubState(TurnSubState::NextAction)
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }
}
