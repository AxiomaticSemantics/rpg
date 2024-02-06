use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Default, Clone, PartialEq, Ser, De)]
pub struct CharacterStatistics {
    pub kills: u32,
    pub times_hit: u32,
    pub attacks: u32,
    pub hits: u32,
    pub dodges: u32,
    pub times_dodged: u32,
    pub blocks: u32,
    pub times_blocked: u32,
    pub chains: u32,
    pub times_chained: u32,
    pub pierced: u32,
    pub times_pierced: u32,
    pub knockbacks: u32,
    pub knockback_distance: f32,
    pub times_knockbacked: u32,
    pub distance_knockbacked: f32,
    pub distance_travelled: f32,
    pub items_looted: u32,

    // Various stats
    pub hp_consumed: u32,
    pub hp_generated: u32,
    pub ep_consumed: u32,
    pub ep_generated: u32,
    pub mp_consumed: u32,
    pub mp_generated: u32,
    pub damage_dealt: u64,
    pub damage_received: u64,
    pub patk_damage_dealt: u64,
    pub matk_damage_dealt: u64,
    pub tatk_damage_dealt: u64,
    pub patk_damage_received: u64,
    pub matk_damage_received: u64,
    pub tatk_damage_received: u64,
}
