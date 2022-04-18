bitflags::bitflags! {
    #[repr(transparent)]
    pub struct AnimationEventFlags : u32 {
        const NONE                       = 0;

        const SOUND_FOOTSTEP             = 1 << 0;
        const SOUND_FOOTSTEP_VEHICLE     = 1 << 1;
        const SOUND_WEAPON_ATTACK_START  = 1 << 2;
        const SOUND_WEAPON_ATTACK_HIT    = 1 << 3;
        const SOUND_WEAPON_FIRE_BULLET   = 1 << 4;
        const SOUND_SKILL_FIRE_BULLET    = 1 << 5;
        const SOUND_SKILL_DUMMY_HIT_0    = 1 << 6;
        const SOUND_SKILL_DUMMY_HIT_1    = 1 << 7;
        const SOUND_SKILL_HIT            = 1 << 8;

        const EFFECT_SKILL_CASTING_0     = 1 << 12;
        const EFFECT_SKILL_CASTING_1     = 1 << 13;
        const EFFECT_SKILL_CASTING_2     = 1 << 14;
        const EFFECT_SKILL_CASTING_3     = 1 << 15;
        const EFFECT_WEAPON_ATTACK_HIT   = 1 << 16;
        const EFFECT_WEAPON_FIRE_BULLET  = 1 << 17;
        const EFFECT_SKILL_FIRE_BULLET   = 1 << 18;
        const EFFECT_SKILL_DUMMY_HIT_0   = 1 << 19;
        const EFFECT_SKILL_DUMMY_HIT_1   = 1 << 20;
        const EFFECT_SKILL_HIT           = 1 << 21;
        const EFFECT_SKILL_ACTION        = 1 << 22;

        const APPLY_RESSURRECTON         = 1 << 29;
        const APPLY_PENDING_SKILL_EFFECT = 1 << 31;
    }
}
