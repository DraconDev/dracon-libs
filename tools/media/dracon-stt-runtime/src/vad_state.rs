#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadState {
    Idle,
    SpeechDetected,
    SpeechConfirmed,
    SilenceDetected,
}

pub struct VadStateMachine {
    pub state: VadState,
    pub speech_frames: usize,
    pub silence_frames: usize,
    pub speech_threshold: f32,
    pub silence_threshold: f32,
    pub min_speech_frames: usize,
    pub min_silence_frames: usize,
}

impl Default for VadStateMachine {
    fn default() -> Self {
        Self {
            state: VadState::Idle,
            speech_frames: 0,
            silence_frames: 0,
            speech_threshold: 0.5,
            silence_threshold: 0.35,
            min_speech_frames: 3,
            min_silence_frames: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadTransition {
    None,
    SpeechStart,
    SpeechEnd,
}

impl VadStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, probability: f32) -> VadTransition {
        match self.state {
            VadState::Idle | VadState::SilenceDetected => {
                if probability > self.speech_threshold {
                    self.speech_frames += 1;
                    if self.speech_frames >= self.min_speech_frames {
                        self.state = VadState::SpeechConfirmed;
                        self.speech_frames = 0;
                        self.silence_frames = 0;
                        return VadTransition::SpeechStart;
                    } else {
                        self.state = VadState::SpeechDetected;
                    }
                } else {
                    self.speech_frames = 0;
                    self.state = VadState::Idle;
                }
            }
            VadState::SpeechDetected => {
                if probability > self.speech_threshold {
                    self.speech_frames += 1;
                    if self.speech_frames >= self.min_speech_frames {
                        self.state = VadState::SpeechConfirmed;
                        self.speech_frames = 0;
                        self.silence_frames = 0;
                        return VadTransition::SpeechStart;
                    }
                } else {
                    self.speech_frames = 0;
                    self.state = VadState::Idle;
                }
            }
            VadState::SpeechConfirmed => {
                if probability < self.silence_threshold {
                    self.silence_frames += 1;
                    if self.silence_frames >= self.min_silence_frames {
                        self.state = VadState::SilenceDetected;
                        self.silence_frames = 0;
                        return VadTransition::SpeechEnd;
                    }
                } else {
                    self.silence_frames = 0;
                }
            }
        }
        VadTransition::None
    }

    pub fn reset(&mut self) {
        self.state = VadState::Idle;
        self.speech_frames = 0;
        self.silence_frames = 0;
    }

    pub fn is_recording(&self) -> bool {
        matches!(
            self.state,
            VadState::SpeechDetected | VadState::SpeechConfirmed
        )
    }

    pub fn state(&self) -> VadState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = VadStateMachine::new();
        assert_eq!(sm.state(), VadState::Idle);
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_speech_detection_sequence() {
        let mut sm = VadStateMachine::new();

        let t1 = sm.process(0.6);
        assert_eq!(t1, VadTransition::None);
        assert_eq!(sm.state(), VadState::SpeechDetected);

        let t2 = sm.process(0.7);
        assert_eq!(t2, VadTransition::None);
        assert!(sm.is_recording());

        let t3 = sm.process(0.8);
        assert_eq!(t3, VadTransition::SpeechStart);
        assert_eq!(sm.state(), VadState::SpeechConfirmed);
    }

    #[test]
    fn test_speech_end_sequence() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        for _ in 0..7 {
            let t = sm.process(0.2);
            assert_eq!(t, VadTransition::None);
        }

        let t = sm.process(0.2);
        assert_eq!(t, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_no_false_positives() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_hysteresis() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        for _ in 0..5 {
            sm.process(0.2);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_reset() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        sm.reset();
        assert_eq!(sm.state, VadState::Idle);
        assert_eq!(sm.speech_frames, 0);
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_is_recording_states() {
        let mut sm = VadStateMachine::new();

        assert!(!sm.is_recording());

        sm.process(0.6);
        assert!(sm.is_recording());

        sm.process(0.6);
        sm.process(0.6);
        assert!(sm.is_recording());

        for _ in 0..8 {
            sm.process(0.2);
        }
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_threshold_boundary() {
        let mut sm = VadStateMachine::new();

        sm.process(0.51);
        sm.process(0.51);
        let t = sm.process(0.51);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_silence_resets_speech_frames() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.2);

        assert_eq!(sm.state, VadState::Idle);
        assert_eq!(sm.speech_frames, 0);
    }

    #[test]
    fn test_vad_state_equality() {
        assert_eq!(VadState::Idle, VadState::Idle);
        assert_eq!(VadState::SpeechDetected, VadState::SpeechDetected);
        assert_ne!(VadState::Idle, VadState::SpeechDetected);
    }

    #[test]
    fn test_vad_state_debug() {
        assert!(format!("{:?}", VadState::Idle).contains("Idle"));
        assert!(format!("{:?}", VadState::SpeechConfirmed).contains("SpeechConfirmed"));
    }

    #[test]
    fn test_vad_transition_equality() {
        assert_eq!(VadTransition::None, VadTransition::None);
        assert_eq!(VadTransition::SpeechStart, VadTransition::SpeechStart);
        assert_eq!(VadTransition::SpeechEnd, VadTransition::SpeechEnd);
        assert_ne!(VadTransition::SpeechStart, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_vad_transition_debug() {
        assert!(format!("{:?}", VadTransition::SpeechStart).contains("SpeechStart"));
    }

    #[test]
    fn test_vad_transition_clone() {
        let t = VadTransition::SpeechStart;
        let cloned = t.clone();
        assert_eq!(t, cloned);
    }

    #[test]
    fn test_vad_transition_copy() {
        let t = VadTransition::SpeechEnd;
        let copied = t;
        assert_eq!(t, copied);
    }

    #[test]
    fn test_default_implementation() {
        let sm1 = VadStateMachine::default();
        let sm2 = VadStateMachine::new();

        assert_eq!(sm1.state, sm2.state);
        assert_eq!(sm1.speech_threshold, sm2.speech_threshold);
    }

    #[test]
    fn test_probability_exactly_at_speech_threshold() {
        let mut sm = VadStateMachine::new();

        let t = sm.process(0.5);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_probability_exac[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSA1ZGgyUVlKV0NkMjVLd3FtL08rZG1BQnlHMUlyQ3l1R0FIazlqcVR0SDJBCjFYbmdtSmRKUjk2ck5rTjBESTJzT09QK0l4KzJhem43Y1N4aXJtbXB4em8KLT4gXy5qTC1ncmVhc2UKWk9YR2h0aVNQelhwOFhwdmlhWmVIUUNRbGpobmlsUzdDV0dUSklobDNBdlVRTDhBNlF6b3NZTzVBK1dwSkdvSQoKLS0tIEFrSW8wSHlvaEIraXNHNG8vckFNTU9wb0lwTUFoUUJKOU1TTU11VXY0ZnMKJTBhEljZ0E4pg84bauii+RkaZWG8QoPyOYJIzyMjb7Fa/5S/MNFHM5SV2oLcO02uSgyRtL0tNbg=]() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        let t = sm.process(0.35);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_probability_just_above_speech_threshold() {
        let mut sm = VadStateMachine::new();

        sm.process(0.5001);
        sm.process(0.5001);
        let t = sm.process(0.5001);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_probability_just_below_silence_threshold() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..7 {
            sm.process(0.3499);
        }
        let t = sm.process(0.3499);
        assert_eq!(t, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_probability_zero() {
        let mut sm = VadStateMachine::new();

        for _ in 0..20 {
            let t = sm.process(0.0);
            assert_eq!(t, VadTransition::None);
        }
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_probability_one() {
        let mut sm = VadStateMachine::new();

        sm.process(1.0);
        sm.process(1.0);
        let t = sm.process(1.0);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_rapid_oscillation() {
        let mut sm = VadStateMachine::new();

        for _ in 0..10 {
            sm.process(0.6);
            sm.process(0.2);
        }

        assert_eq!(sm.state, VadState::Idle);
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_very_long_speech() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for i in 0..1000 {
            let t = sm.process(0.8);
            assert_eq!(t, VadTransition::None);
            assert!(sm.is_recording());

            if i % 100 == 0 {
                assert_eq!(sm.state, VadState::SpeechConfirmed);
            }
        }
    }

    #[test]
    fn test_reset_during_speech() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        sm.reset();

        assert_eq!(sm.state, VadState::Idle);
        assert!(!sm.is_recording());
        assert_eq!(sm.speech_frames, 0);
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_speech_after_reset() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.reset();

        sm.process(0.6);
        sm.process(0.6);
        let t = sm.process(0.6);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_multiple_speech_cycles() {
        let mut sm = VadStateMachine::new();

        for _ in 0..5 {
            for _ in 0..3 {
                sm.process(0.6);
            }
            assert_eq!(sm.state, VadState::SpeechConfirmed);

            for _ in 0..8 {
                sm.process(0.2);
            }
            assert_eq!(sm.state, VadState::SilenceDetected);

            sm.process(0.2);
            assert_eq!(sm.state, VadState::Idle);
        }
    }

    #[test]
    fn test_is_recording_all_states() {
        let mut sm = VadStateMachine::new();

        assert!(!sm.is_recording());

        sm.process(0.6);
        assert!(sm.is_recording());

        sm.process(0.6);
        sm.process(0.6);
        assert!(sm.is_recording());

        for _ in 0..8 {
            sm.process(0.2);
        }
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_exactly_min_speech_frames() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        sm.process(0.6);
        let t = sm.process(0.6);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_one_less_than_min_speech_frames() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        let t = sm.process(0.2);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_exactly_min_silence_frames() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..7 {
            sm.process(0.2);
        }
        let t = sm.process(0.2);
        assert_eq!(t, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_one_less_than_min_silence_frames() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..7 {
            sm.process(0.2);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_interrupted_silence_counting() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..5 {
            sm.process(0.2);
        }
        assert!(sm.silence_frames > 0);

        sm.process(0.6);
        assert_eq!(sm.silence_frames, 0);

        for _ in 0..7 {
            sm.process(0.2);
        }
        let t = sm.process(0.2);
        assert_eq!(t, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_vad_state_copy() {
        let state = VadState::SpeechConfirmed;
        let copied = state;
        assert_eq!(state, copied);
    }

    #[test]
    fn test_vad_state_clone() {
        let state = VadState::SpeechDetected;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_negative_probability() {
        let mut sm = VadStateMachine::new();

        let t = sm.process(-0.5);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_probability_above_one() {
        let mut sm = VadStateMachine::new();

        sm.process(1.5);
        sm.process(1.5);
        let t = sm.process(1.5);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_nan_probability() {
        let mut sm = VadStateMachine::new();

        let t = sm.process(f32::NAN);
        assert_eq!(t, VadTransition::None);
    }

    #[test]
    fn test_infinity_probability() {
        let mut sm = VadStateMachine::new();

        sm.process(f32::INFINITY);
        sm.process(f32::INFINITY);
        let t = sm.process(f32::INFINITY);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_vad_state_all_variants() {
        let states = [
            VadState::Idle,
            VadState::SpeechDetected,
            VadState::SpeechConfirmed,
            VadState::SilenceDetected,
        ];
        assert_eq!(states.len(), 4);
    }

    #[test]
    fn test_vad_transition_all_variants() {
        let transitions = [
            VadTransition::None,
            VadTransition::SpeechStart,
            VadTransition::SpeechEnd,
        ];
        assert_eq!(transitions.len(), 3);
    }

    #[test]
    fn test_silence_detected_transition() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }

        assert_eq!(sm.state, VadState::SilenceDetected);
    }

    #[test]
    fn test_silence_detected_stays_idle_after_silence() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        sm.process(0.2);

        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_speech_detected_partial() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
    }

    #[test]
    fn test_speech_detected_to_confirmed() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_speech_detected_to_idle() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_silence_frames_reset_on_speech() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..10 {
            sm.process(0.2);
        }
        sm.process(0.6);

        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_speech_frames_reset_on_silence() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        assert_eq!(sm.speech_frames, 1);
        sm.process(0.2);
        assert_eq!(sm.speech_frames, 0);
    }

    #[test]
    fn test_exact_threshold_speech() {
        let mut sm = VadStateMachine::new();

        let t = sm.process(0.500001);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::SpeechDetected);
    }

    #[test]
    fn test_exact_threshold_silence() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        let t = sm.process(0.349999);
        assert_eq!(t, VadTransition::None);
    }

    #[test]
    fn test_just_below_speech_threshold() {
        let mut sm = VadStateMachine::new();

        let t = sm.process(0.499999);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_just_above_silence_threshold() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..20 {
            let t = sm.process(0.350001);
            assert_eq!(t, VadTransition::None);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_vad_transition_ordering() {
        let mut sm = VadStateMachine::new();

        assert_eq!(sm.process(0.6), VadTransition::None);
        assert_eq!(sm.process(0.6), VadTransition::None);
        assert_eq!(sm.process(0.6), VadTransition::SpeechStart);

        for _ in 0..7 {
            assert_eq!(sm.process(0.2), VadTransition::None);
        }
        assert_eq!(sm.process(0.2), VadTransition::SpeechEnd);
    }

    #[test]
    fn test_is_recording_speech_detected() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        assert!(sm.is_recording());
    }

    #[test]
    fn test_is_recording_speech_confirmed() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        assert!(sm.is_recording());
    }

    #[test]
    fn test_is_recording_silence_detected() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_probability_very_small_positive() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(1e-10);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_probability_very_small_negative() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(-1e-10);
        assert_eq!(t, VadTransition::None);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_negative_infinity() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(f32::NEG_INFINITY);
        assert_eq!(t, VadTransition::None);
    }

    #[test]
    fn test_multiple_resets() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.reset();

        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.reset();

        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.reset();

        assert_eq!(sm.state, VadState::Idle);
        assert_eq!(sm.speech_frames, 0);
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_oscillation_at_boundary() {
        let mut sm = VadStateMachine::new();

        for _ in 0..100 {
            sm.process(0.51);
            sm.process(0.49);
        }

        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_long_silence_before_speech() {
        let mut sm = VadStateMachine::new();

        for _ in 0..1000 {
            sm.process(0.1);
        }
        assert_eq!(sm.state, VadState::Idle);

        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_speech_without_end() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..100 {
            sm.process(0.7);
        }

        assert_eq!(sm.state, VadState::SpeechConfirmed);
        assert!(sm.is_recording());
    }

    #[test]
    fn test_vad_state_ne_values() {
        assert_ne!(VadState::Idle, VadState::SpeechConfirmed);
        assert_ne!(VadState::SpeechDetected, VadState::SilenceDetected);
        assert_ne!(VadState::Idle, VadState::SpeechDetected);
    }

    #[test]
    fn test_vad_transition_ne_values() {
        assert_ne!(VadTransition::None, VadTransition::SpeechStart);
        assert_ne!(VadTransition::SpeechStart, VadTransition::SpeechEnd);
        assert_ne!(VadTransition::None, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_immediate_silence_after_speech_start() {
        let mut sm = VadStateMachine::new();

        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        for _ in 0..8 {
            sm.process(0.2);
        }
        assert_eq!(sm.state, VadState::SilenceDetected);
    }

    #[test]
    fn test_speech_restart_after_silence() {
        let mut sm = VadStateMachine::new();

        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);

        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_partial_speech_repeated() {
        let mut sm = VadStateMachine::new();

        for _ in 0..10 {
            sm.process(0.6);
            sm.process(0.6);
            sm.process(0.2);
        }

        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_exac[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBVb2pMYzFxbjQxUVI1M1IvV1ZzaUMzZktSdEVsbW1EWmtWUkNKNWNJYlFNCnhQZEUrbE5XS1hCVGhEcldlMEtiYmNmc09FVEthYXJrdlNpTG5sblZQKzAKLT4gJTl3eC1ncmVhc2UgMS9aYn0gTiA7Ck9VNzNKYTNkNk5nTG95SS9MQlRxYk9Eb1ZESWJ2OEEKLS0tIEtjWHdkTE5BUkhJdUpXc0lMT2NOUW5iUWhCNE1SRno2UlYwVGF1YzRxa1EKKeVdTYE/4S57OofvAx8ieDVwQv+7T+yJvFWgTZuEj17RzIJyZOV0EZgGeqFbHe1Nu7goUpSL1Fg=]() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_exactly_min_frames() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(0.6);
        assert_eq!(t, VadTransition::None);
        let t = sm.process(0.6);
        assert_eq!(t, VadTransition::None);
        let t = sm.process(0.6);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_more_than_min_frames() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_silence_frames_exactly_min() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..7 {
            sm.process(0.2);
        }
        assert_eq!(sm.state, VadState::SpeechConfirmed);

        let t = sm.process(0.2);
        assert_eq!(t, VadTransition::SpeechEnd);
    }

    #[test]
    fn test_subnormal_probability() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(f32::MIN_POSITIVE);
        assert_eq!(t, VadTransition::None);
    }

    #[test]
    fn test_large_negative_probability() {
        let mut sm = VadStateMachine::new();
        let t = sm.process(-1000.0);
        assert_eq!(t, VadTransition::None);
    }

    #[test]
    fn test_large_positive_probability() {
        let mut sm = VadStateMachine::new();
        sm.process(1000.0);
        sm.process(1000.0);
        let t = sm.process(1000.0);
        assert_eq!(t, VadTransition::SpeechStart);
    }

    #[test]
    fn test_vad_state_debug_format() {
        assert!(format!("{:?}", VadState::Idle).contains("Idle"));
        assert!(format!("{:?}", VadState::SpeechDetected).contains("SpeechDetected"));
        assert!(format!("{:?}", VadState::SpeechConfirmed).contains("SpeechConfirmed"));
        assert!(format!("{:?}", VadState::SilenceDetected).contains("SilenceDetected"));
    }

    #[test]
    fn test_vad_transition_debug_format() {
        assert!(format!("{:?}", VadTransition::None).contains("None"));
        assert!(format!("{:?}", VadTransition::SpeechStart).contains("SpeechStart"));
        assert!(format!("{:?}", VadTransition::SpeechEnd).contains("SpeechEnd"));
    }

    #[test]
    fn test_vad_state_partial_eq() {
        assert!(VadState::Idle == VadState::Idle);
        assert!(VadState::SpeechDetected == VadState::SpeechDetected);
        assert!(VadState::SpeechConfirmed == VadState::SpeechConfirmed);
        assert!(VadState::SilenceDetected == VadState::SilenceDetected);
    }

    #[test]
    fn test_vad_transition_partial_eq() {
        assert!(VadTransition::None == VadTransition::None);
        assert!(VadTransition::SpeechStart == VadTransition::SpeechStart);
        assert!(VadTransition::SpeechEnd == VadTransition::SpeechEnd);
    }

    #[test]
    fn test_vad_transition_eq_trait() {
        let t1 = VadTransition::SpeechStart;
        let t2 = VadTransition::SpeechStart;
        assert!(t1 == t2);
    }

    #[test]
    fn test_vad_state_from_idle_to_speech_detected() {
        let mut sm = VadStateMachine::new();
        assert_eq!(sm.state, VadState::Idle);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechDetected);
    }

    #[test]
    fn test_vad_state_from_speech_detected_to_confirmed() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_vad_state_from_confirmed_to_silence() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        assert_eq!(sm.state, VadState::SilenceDetected);
    }

    #[test]
    fn test_vad_state_from_silence_to_idle() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        sm.process(0.2);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_vad_state_idle_stays_idle_on_silence() {
        let mut sm = VadStateMachine::new();
        for _ in 0..100 {
            sm.process(0.1);
            assert_eq!(sm.state, VadState::Idle);
        }
    }

    #[test]
    fn test_vad_state_speech_frames_increment() {
        let mut sm = VadStateMachine::new();
        assert_eq!(sm.speech_frames, 0);
        sm.process(0.6);
        assert_eq!(sm.speech_frames, 1);
        sm.process(0.6);
        assert_eq!(sm.speech_frames, 2);
    }

    #[test]
    fn test_vad_state_silence_frames_increment() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        assert_eq!(sm.silence_frames, 0);
        sm.process(0.2);
        assert_eq!(sm.silence_frames, 1);
        sm.process(0.2);
        assert_eq!(sm.silence_frames, 2);
    }

    #[test]
    fn test_vad_threshold_default_values() {
        let sm = VadStateMachine::new();
        assert_eq!(sm.speech_threshold, 0.5);
        assert_eq!(sm.silence_threshold, 0.35);
        assert_eq!(sm.min_speech_frames, 3);
        assert_eq!(sm.min_silence_frames, 8);
    }

    #[test]
    fn test_vad_state_is_recording_idle() {
        let sm = VadStateMachine::new();
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_vad_state_is_recording_speech_detected() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        assert!(sm.is_recording());
    }

    #[test]
    fn test_vad_state_is_recording_speech_confirmed() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        assert!(sm.is_recording());
    }

    #[test]
    fn test_vad_state_is_recording_silence_detected() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..8 {
            sm.process(0.2);
        }
        assert!(!sm.is_recording());
    }

    #[test]
    fn test_vad_process_returns_transition() {
        let mut sm = VadStateMachine::new();

        let t1 = sm.process(0.6);
        assert!(matches!(t1, VadTransition::None));

        let t2 = sm.process(0.6);
        assert!(matches!(t2, VadTransition::None));

        let t3 = sm.process(0.6);
        assert!(matches!(t3, VadTransition::SpeechStart));
    }

    #[test]
    fn test_vad_reset_clears_speech_frames() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        sm.process(0.6);
        assert_eq!(sm.speech_frames, 2);
        sm.reset();
        assert_eq!(sm.speech_frames, 0);
    }

    #[test]
    fn test_vad_reset_clears_silence_frames() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..5 {
            sm.process(0.2);
        }
        assert!(sm.silence_frames > 0);
        sm.reset();
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_vad_probability_edge_0_49() {
        let mut sm = VadStateMachine::new();
        sm.process(0.49);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_vad_probability_edge_0_50() {
        let mut sm = VadStateMachine::new();
        sm.process(0.50);
        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_vad_probability_edge_0_51() {
        let mut sm = VadStateMachine::new();
        sm.process(0.51);
        assert_eq!(sm.state, VadState::SpeechDetected);
    }

    #[test]
    fn test_vad_silence_edge_0_34() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.process(0.34);
        assert_eq!(sm.silence_frames, 1);
    }

    #[test]
    fn test_vad_silence_edge_0_35() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.process(0.35);
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_vad_silence_edge_0_36() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        sm.process(0.36);
        assert_eq!(sm.silence_frames, 0);
    }

    #[test]
    fn test_vad_continuous_speech_no_end() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..1000 {
            let t = sm.process(0.8);
            assert!(matches!(t, VadTransition::None));
        }

        assert_eq!(sm.state, VadState::SpeechConfirmed);
    }

    #[test]
    fn test_vad_continuous_silence_no_start() {
        let mut sm = VadStateMachine::new();

        for _ in 0..1000 {
            let t = sm.process(0.1);
            assert!(matches!(t, VadTransition::None));
        }

        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_vad_alternating_probabilities() {
        let mut sm = VadStateMachine::new();

        for _ in 0..100 {
            sm.process(0.6);
            sm.process(0.3);
            sm.process(0.6);
            sm.process(0.3);
        }

        assert_eq!(sm.state, VadState::Idle);
    }

    #[test]
    fn test_vad_transition_order_speech() {
        let mut sm = VadStateMachine::new();

        assert!(matches!(sm.process(0.6), VadTransition::None));
        assert!(matches!(sm.process(0.6), VadTransition::None));
        assert!(matches!(sm.process(0.6), VadTransition::SpeechStart));
    }

    #[test]
    fn test_vad_transition_order_silence() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }

        for _ in 0..7 {
            assert!(matches!(sm.process(0.2), VadTransition::None));
        }
        assert!(matches!(sm.process(0.2), VadTransition::SpeechEnd));
    }

    #[test]
    fn test_vad_speech_frames_reset_on_silence_during_detected() {
        let mut sm = VadStateMachine::new();
        sm.process(0.6);
        assert_eq!(sm.speech_frames, 1);
        sm.process(0.2);
        assert_eq!(sm.speech_frames, 0);
    }

    #[test]
    fn test_vad_silence_frames_reset_on_speech_during_confirmed() {
        let mut sm = VadStateMachine::new();
        for _ in 0..3 {
            sm.process(0.6);
        }
        for _ in 0..5 {
            sm.process(0.2);
        }
        assert_eq!(sm.silence_frames, 5);
        sm.process(0.6);
        assert_eq!(sm.silence_frames, 0);
    }
}
