use std::{collections::BTreeMap, fmt};

use serde::Serialize;

use crate::world::{environment::Environment, transmission_status::TransmissionStatus};

/// Observed summary of an environment: who is present, and what was
/// emitted, delivered or could not be delivered, between whom, over an
/// optional period of environment time. Counts only; nothing is inferred.
#[derive(Debug, Serialize)]
pub(crate) struct EnvironmentReport {
    name: String,
    clock: f64,
    latency: f64,
    perceptibility: f64,
    external: bool,
    members: Vec<(String, String, f64, bool)>,
    period: (f64, f64),
    emitted: BTreeMap<String, u64>,
    delivered: BTreeMap<String, u64>,
    pending: u64,
    undeliverable: u64,
    statements: u64,
    questions: u64,
    relayed: u64,
    documents: u64,
}

impl EnvironmentReport {
    pub(crate) fn of(environment: &Environment, from: f64, to: f64) -> Self {
        let mut report = Self {
            name: environment.name().to_owned(),
            clock: environment.clock(),
            latency: environment.latency(),
            perceptibility: environment.perceptibility(),
            external: environment.external(),
            members: environment
                .members()
                .iter()
                .map(|member| {
                    (
                        member.handle().to_owned(),
                        member.individual().to_owned(),
                        member.joined_at(),
                        member.is_present(),
                    )
                })
                .collect(),
            period: (from, to),
            emitted: BTreeMap::new(),
            delivered: BTreeMap::new(),
            pending: 0,
            undeliverable: 0,
            statements: 0,
            questions: 0,
            relayed: 0,
            documents: 0,
        };

        for transmission in environment.transmissions() {
            if transmission.emitted_at() < from || transmission.emitted_at() > to {
                continue;
            }

            let pair = format!("{} -> {}", transmission.from(), transmission.to());

            *report.emitted.entry(pair.clone()).or_default() += 1;

            match transmission.status() {
                TransmissionStatus::Pending => report.pending += 1,
                TransmissionStatus::Delivered { .. } => {
                    *report.delivered.entry(pair).or_default() += 1;
                }
                TransmissionStatus::Undeliverable { .. } => report.undeliverable += 1,
            }

            let message = transmission.message();

            report.statements += message.statements().len() as u64;
            report.questions += message.questions().len() as u64;
            report.relayed += message.relayed().len() as u64;
            report.documents += message.documents().len() as u64;
        }

        report
    }
}

impl fmt::Display for EnvironmentReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "{}: environment time {:.0} s, latency {:.0} s, perceptibility {}, external access {}",
            self.name, self.clock, self.latency, self.perceptibility, self.external
        )?;

        for (handle, individual, joined, present) in &self.members {
            writeln!(
                formatter,
                "  {handle}  {individual}  joined at {joined:.0} s{}",
                if *present { "" } else { " (departed)" }
            )?;
        }

        writeln!(
            formatter,
            "transmissions emitted in [{:.0}, {:.0}] s: {} ({} pending, {} undeliverable); content: {} statements, {} questions, {} relayed, {} texts",
            self.period.0,
            self.period.1,
            self.emitted.values().sum::<u64>(),
            self.pending,
            self.undeliverable,
            self.statements,
            self.questions,
            self.relayed,
            self.documents
        )?;

        for (pair, count) in &self.emitted {
            writeln!(
                formatter,
                "  {pair}: {count} emitted, {} delivered",
                self.delivered.get(pair).copied().unwrap_or(0)
            )?;
        }

        Ok(())
    }
}
