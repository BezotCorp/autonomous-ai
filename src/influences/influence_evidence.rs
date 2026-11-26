use serde::Serialize;

/// What is known of the effects of an influence in reality, from the
/// weakest to the strongest form of evidence.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(crate) struct InfluenceEvidence {
    /// Real episodes in which it was present (correlation only).
    exposures: u64,
    /// Decisions it changed, and among them those whose outcome was a
    /// verified success or not (observed contribution).
    decisive_successes: u64,
    decisive_failures: u64,
    /// Its content supported an answer, or was refuted.
    supported: u64,
    refuted: u64,
    /// Replays without it that changed, or did not change, the outcome
    /// (demonstrated attribution).
    demonstrated: u64,
    not_demonstrated: u64,
}

impl InfluenceEvidence {
    pub(crate) fn expose(&mut self) {
        self.exposures += 1;
    }

    pub(crate) fn decide(&mut self, success: bool) {
        if success {
            self.decisive_successes += 1;
        } else {
            self.decisive_failures += 1;
        }
    }

    pub(crate) fn support(&mut self) {
        self.supported += 1;
    }

    pub(crate) fn refute(&mut self) {
        self.refuted += 1;
    }

    pub(crate) fn demonstrate(&mut self, made_a_difference: bool) {
        if made_a_difference {
            self.demonstrated += 1;
        } else {
            self.not_demonstrated += 1;
        }
    }

    pub(crate) fn exposures(&self) -> u64 {
        self.exposures
    }

    /// Outcomes of the decisions it changed, counting its content
    /// supporting an answer as a success and its refutation as a failure.
    pub(crate) fn outcomes(&self) -> (u64, u64) {
        (
            self.decisive_successes + self.supported,
            self.decisive_failures + self.refuted,
        )
    }

    pub(crate) fn demonstrations(&self) -> (u64, u64) {
        (self.demonstrated, self.not_demonstrated)
    }

    /// The strongest form of evidence available.
    pub(crate) fn attribution(&self) -> &'static str {
        if self.demonstrated + self.not_demonstrated > 0 {
            "demonstrated"
        } else if self.decisive_successes + self.decisive_failures + self.supported + self.refuted
            > 0
        {
            "observed_contribution"
        } else if self.exposures > 0 {
            "correlation"
        } else {
            "untested"
        }
    }
}
