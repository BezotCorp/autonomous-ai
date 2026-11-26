use std::time::Instant;

/// Progress of one living session: where it began, which scheduled stimuli
/// were delivered, whose turn it is among the activities under way, and
/// when the state was last sampled and saved. A session may be lived in
/// one go or window by window (in a shared environment).
pub(crate) struct Session {
    start: f64,
    end: f64,
    last_checkpoint: Instant,
    next_sample: f64,
    next_plasticity: f64,
    scheduled: usize,
    turn: usize,
    stopped: bool,
}

impl Session {
    pub(crate) fn new(start: f64, duration: f64) -> Self {
        Self {
            start,
            end: start + duration.max(0.0),
            last_checkpoint: Instant::now(),
            next_sample: start,
            next_plasticity: start,
            scheduled: 0,
            turn: 0,
            stopped: false,
        }
    }

    pub(crate) fn start(&self) -> f64 {
        self.start
    }

    pub(crate) fn end(&self) -> f64 {
        self.end
    }

    pub(crate) fn scheduled(&self) -> usize {
        self.scheduled
    }

    pub(crate) fn delivered_scheduled(&mut self) {
        self.scheduled += 1;
    }

    pub(crate) fn next_turn(&mut self) -> usize {
        let turn = self.turn;

        self.turn = self.turn.wrapping_add(1);
        turn
    }

    pub(crate) fn sample_due(&mut self, now: f64, every: f64) -> bool {
        if now >= self.next_sample {
            self.next_sample = now + every;
            true
        } else {
            false
        }
    }

    pub(crate) fn plasticity_due(&mut self, now: f64, every: f64) -> bool {
        if now >= self.next_plasticity {
            self.next_plasticity = now + every;
            true
        } else {
            false
        }
    }

    pub(crate) fn checkpoint_due(&mut self, every: f64) -> bool {
        if self.last_checkpoint.elapsed().as_secs_f64() >= every {
            self.last_checkpoint = Instant::now();
            true
        } else {
            false
        }
    }

    pub(crate) fn stop(&mut self) {
        self.stopped = true;
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.stopped
    }
}
