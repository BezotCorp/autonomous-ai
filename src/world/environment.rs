use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    mathematics::stable_hash::StableHash,
    world::{
        member::Member, message::Message, transmission::Transmission,
        transmission_status::TransmissionStatus,
    },
};

/// What exists around some existences: who is present, the messages on
/// their way between them, and the physical and operational possibilities
/// the place offers (how long a message takes, how perceptible a presence
/// is, whether external systems can be reached). It decides nothing for
/// anyone: existences perceive it and act in it through their own
/// capabilities, or never do.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Environment {
    id: Uuid,
    name: String,
    seed: u64,
    created_at: DateTime<Utc>,
    clock: f64,
    latency: f64,
    perceptibility: f64,
    external: bool,
    members: Vec<Member>,
    transmissions: Vec<Transmission>,
}

impl Environment {
    /// Shortest latency: the environment advances existences by windows of
    /// at most the latency, so that nothing is received before it is sent.
    pub(crate) const MINIMUM_LATENCY: f64 = 1.0;

    pub(crate) fn new(
        name: String,
        seed: u64,
        latency: f64,
        perceptibility: f64,
        external: bool,
    ) -> Result<Self, String> {
        if !(latency.is_finite() && latency >= Self::MINIMUM_LATENCY) {
            return Err(format!(
                "the latency must be at least {} s",
                Self::MINIMUM_LATENCY
            ));
        }

        if !(0.0..=1.0).contains(&perceptibility) {
            return Err("the perceptibility is a probability, in [0, 1]".to_owned());
        }

        Ok(Self {
            id: Uuid::now_v7(),
            name,
            seed,
            created_at: Utc::now(),
            clock: 0.0,
            latency,
            perceptibility,
            external,
            members: Vec::new(),
            transmissions: Vec::new(),
        })
    }

    /// An environment as it was saved (format migrations).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn restore(
        id: Uuid,
        name: String,
        seed: u64,
        created_at: DateTime<Utc>,
        clock: f64,
        latency: f64,
        perceptibility: f64,
        external: bool,
        members: Vec<Member>,
        transmissions: Vec<Transmission>,
    ) -> Self {
        Self {
            id,
            name,
            seed,
            created_at,
            clock,
            latency,
            perceptibility,
            external,
            members,
            transmissions,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn clock(&self) -> f64 {
        self.clock
    }

    pub(crate) fn advance_to(&mut self, time: f64) {
        if time.is_finite() && time > self.clock {
            self.clock = time;
        }
    }

    pub(crate) fn latency(&self) -> f64 {
        self.latency
    }

    pub(crate) fn perceptibility(&self) -> f64 {
        self.perceptibility
    }

    /// Whether external systems (the network) can be reached from here.
    pub(crate) fn external(&self) -> bool {
        self.external
    }

    pub(crate) fn members(&self) -> &[Member] {
        &self.members
    }

    pub(crate) fn transmissions(&self) -> &[Transmission] {
        &self.transmissions
    }

    /// Adds an existence under a new handle, derived from the environment
    /// seed and the number of members (reproducible, never a name).
    pub(crate) fn admit(&mut self, individual: String) -> Result<String, String> {
        if self
            .members
            .iter()
            .any(|member| member.is_present() && member.individual() == individual)
        {
            return Err(format!("{individual} is already present"));
        }

        let handle = format!(
            "e{:08x}",
            StableHash::of(self.seed, &format!("member:{}", self.members.len())) as u32
        );

        self.members
            .push(Member::new(handle.clone(), individual, self.clock));
        Ok(handle)
    }

    pub(crate) fn remove(&mut self, individual: &str) -> Result<(), String> {
        let clock = self.clock;
        let member = self
            .members
            .iter_mut()
            .find(|member| member.is_present() && member.individual() == individual)
            .ok_or(format!("{individual} is not present"))?;

        member.depart(clock);
        Ok(())
    }

    /// Present members other than `observer` that happen to be perceptible
    /// at `time`: each presence is perceived with the environment's
    /// perceptibility, by a draw reproducible from the seed, the observer,
    /// the observed and the time.
    pub(crate) fn perceptible(&self, observer: &str, time: f64) -> Vec<String> {
        self.members
            .iter()
            .filter(|member| member.is_present() && member.handle() != observer)
            .filter(|member| {
                let draw = StableHash::of(
                    self.seed,
                    &format!("perceive:{observer}:{}:{}", member.handle(), time.to_bits()),
                );

                ((draw >> 11) as f64 / (1_u64 << 53) as f64) < self.perceptibility
            })
            .map(|member| member.handle().to_owned())
            .collect()
    }

    /// Sends a message; it is pending if the destination is present here,
    /// undeliverable otherwise (and recorded as such).
    pub(crate) fn transmit(
        &mut self,
        from: &str,
        to: &str,
        time: f64,
        message: Message,
    ) -> (Uuid, TransmissionStatus) {
        let reachable = to != from
            && self
                .members
                .iter()
                .any(|member| member.is_present() && member.handle() == to);
        let status = if reachable {
            TransmissionStatus::Pending
        } else {
            TransmissionStatus::Undeliverable {
                reason: format!("no one reachable at {to}"),
            }
        };
        let id = Uuid::now_v7();

        self.transmissions.push(Transmission::new(
            id,
            from.to_owned(),
            to.to_owned(),
            time,
            time + self.latency,
            message,
            status.clone(),
        ));
        (id, status)
    }

    pub(crate) fn next_arrival(&self, recipient: &str) -> Option<f64> {
        self.transmissions
            .iter()
            .filter(|transmission| {
                transmission.to() == recipient
                    && transmission.status() == &TransmissionStatus::Pending
            })
            .map(Transmission::deliver_at)
            .min_by(f64::total_cmp)
    }

    /// Transmissions for `recipient` that can be received at `time`, in
    /// the order they were sent; they are marked delivered.
    pub(crate) fn deliver(&mut self, recipient: &str, time: f64) -> Vec<Transmission> {
        let mut delivered = Vec::new();

        for transmission in &mut self.transmissions {
            if transmission.to() == recipient
                && transmission.status() == &TransmissionStatus::Pending
                && transmission.deliver_at() <= time
            {
                transmission.deliver(time);
                delivered.push(transmission.clone());
            }
        }

        delivered
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;
    use crate::world::{message::Message, transmission_status::TransmissionStatus};

    #[test]
    fn nothing_is_received_before_it_is_sent_plus_the_latency() {
        let mut environment = Environment::new("test".to_owned(), 1, 60.0, 1.0, false).unwrap();
        let first = environment.admit("a".to_owned()).unwrap();
        let second = environment.admit("b".to_owned()).unwrap();

        assert_ne!(first, second);
        assert!(environment.admit("a".to_owned()).is_err());
        assert_eq!(environment.perceptible(&first, 0.0), vec![second.clone()]);

        let (_, status) = environment.transmit(
            &first,
            &second,
            100.0,
            Message::new(Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        );

        assert_eq!(status, TransmissionStatus::Pending);
        assert!(environment.deliver(&second, 159.0).is_empty());
        assert_eq!(environment.deliver(&second, 160.0).len(), 1);
        // Delivered once only.
        assert!(environment.deliver(&second, 1_000.0).is_empty());

        let (_, status) = environment.transmit(&first, "nobody", 200.0, Message::default());

        assert!(matches!(status, TransmissionStatus::Undeliverable { .. }));
        assert!(Environment::new("x".to_owned(), 1, 0.5, 1.0, false).is_err());

        let unseen = Environment::new("y".to_owned(), 1, 60.0, 0.0, false).unwrap();

        assert!(unseen.perceptible(&first, 0.0).is_empty());
    }
}
