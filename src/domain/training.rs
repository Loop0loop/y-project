#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdvocateStats {
    pub(crate) logic_speed: u16,
    pub(crate) mental_stamina: u16,
    pub(crate) speech_power: u16,
    pub(crate) guts: u16,
    pub(crate) intellect: u16,
}

impl AdvocateStats {
    pub(super) fn total(self) -> u16 {
        self.logic_speed + self.mental_stamina + self.speech_power + self.guts + self.intellect
    }
}

impl Default for AdvocateStats {
    fn default() -> Self {
        Self {
            logic_speed: 28,
            mental_stamina: 35,
            speech_power: 32,
            guts: 24,
            intellect: 30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrainingActionId {
    LogicDrill,
    SpeechPractice,
    LawStudy,
    NerveControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct AdvocateStatsDelta {
    logic_speed: i16,
    mental_stamina: i16,
    speech_power: i16,
    guts: i16,
    intellect: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrainingAction {
    pub(crate) id: TrainingActionId,
    pub(crate) label: &'static str,
    pub(super) delta: AdvocateStatsDelta,
}

pub(crate) const TRAINING_ACTIONS: [TrainingAction; 4] = [
    TrainingAction {
        id: TrainingActionId::LogicDrill,
        label: "Logic Drill",
        delta: AdvocateStatsDelta {
            logic_speed: 12,
            ..AdvocateStatsDelta::ZERO
        },
    },
    TrainingAction {
        id: TrainingActionId::SpeechPractice,
        label: "Speech Practice",
        delta: AdvocateStatsDelta {
            speech_power: 12,
            ..AdvocateStatsDelta::ZERO
        },
    },
    TrainingAction {
        id: TrainingActionId::LawStudy,
        label: "Law Study",
        delta: AdvocateStatsDelta {
            intellect: 12,
            ..AdvocateStatsDelta::ZERO
        },
    },
    TrainingAction {
        id: TrainingActionId::NerveControl,
        label: "Nerve Control",
        delta: AdvocateStatsDelta {
            mental_stamina: 8,
            guts: 6,
            ..AdvocateStatsDelta::ZERO
        },
    },
];

impl AdvocateStatsDelta {
    const ZERO: Self = Self {
        logic_speed: 0,
        mental_stamina: 0,
        speech_power: 0,
        guts: 0,
        intellect: 0,
    };
}

pub(super) fn apply_delta(stats: AdvocateStats, delta: AdvocateStatsDelta) -> AdvocateStats {
    fn add(value: u16, delta: i16) -> u16 {
        (value as i16 + delta).clamp(0, 100) as u16
    }

    AdvocateStats {
        logic_speed: add(stats.logic_speed, delta.logic_speed),
        mental_stamina: add(stats.mental_stamina, delta.mental_stamina),
        speech_power: add(stats.speech_power, delta.speech_power),
        guts: add(stats.guts, delta.guts),
        intellect: add(stats.intellect, delta.intellect),
    }
}
