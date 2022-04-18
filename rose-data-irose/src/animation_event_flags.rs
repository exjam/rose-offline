use rose_data::AnimationEventFlags;

pub fn get_animation_event_flags() -> Vec<AnimationEventFlags> {
    let mut animation_event_flags = vec![AnimationEventFlags::NONE; 100];

    animation_event_flags[10] = AnimationEventFlags::EFFECT_SKILL_DUMMY_HIT_0
        | AnimationEventFlags::SOUND_SKILL_DUMMY_HIT_0;

    animation_event_flags[20] = AnimationEventFlags::EFFECT_SKILL_DUMMY_HIT_1
        | AnimationEventFlags::SOUND_SKILL_DUMMY_HIT_1;

    animation_event_flags[21] = AnimationEventFlags::EFFECT_WEAPON_ATTACK_HIT
        | AnimationEventFlags::SOUND_WEAPON_ATTACK_HIT;

    animation_event_flags[22] = AnimationEventFlags::EFFECT_WEAPON_FIRE_BULLET
        | AnimationEventFlags::SOUND_WEAPON_FIRE_BULLET;

    animation_event_flags[23] = AnimationEventFlags::EFFECT_WEAPON_FIRE_BULLET
        | AnimationEventFlags::SOUND_WEAPON_FIRE_BULLET;

    animation_event_flags[24] = AnimationEventFlags::APPLY_PENDING_SKILL_EFFECT
        | AnimationEventFlags::SOUND_SKILL_FIRE_BULLET
        | AnimationEventFlags::EFFECT_SKILL_ACTION;

    animation_event_flags[25] = AnimationEventFlags::APPLY_PENDING_SKILL_EFFECT
        | AnimationEventFlags::EFFECT_SKILL_HIT
        | AnimationEventFlags::SOUND_SKILL_HIT
        | AnimationEventFlags::EFFECT_SKILL_ACTION;

    animation_event_flags[26] = AnimationEventFlags::EFFECT_SKILL_FIRE_BULLET;

    animation_event_flags[31] = AnimationEventFlags::SOUND_WEAPON_ATTACK_START;

    animation_event_flags[32] = AnimationEventFlags::SOUND_WEAPON_ATTACK_START;

    animation_event_flags[33] = AnimationEventFlags::SOUND_WEAPON_ATTACK_START;

    animation_event_flags[34] = AnimationEventFlags::SOUND_SKILL_FIRE_BULLET;

    animation_event_flags[44] = AnimationEventFlags::EFFECT_SKILL_CASTING_0;

    animation_event_flags[56] = AnimationEventFlags::EFFECT_SKILL_FIRE_DUMMY_BULLET;

    animation_event_flags[64] = AnimationEventFlags::EFFECT_SKILL_CASTING_1;

    animation_event_flags[66] = AnimationEventFlags::EFFECT_SKILL_FIRE_DUMMY_BULLET;

    animation_event_flags[74] = AnimationEventFlags::EFFECT_SKILL_CASTING_2;

    animation_event_flags[84] = AnimationEventFlags::EFFECT_SKILL_CASTING_3;

    animation_event_flags[91] = AnimationEventFlags::APPLY_RESSURRECTON;

    animation_event_flags
}
