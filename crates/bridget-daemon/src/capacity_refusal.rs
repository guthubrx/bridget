//! Preuve fermée du seul refus de capacité déjà évalué. Aucun sondage ni effet.
use serde_json::{Value, json};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapacityRefusal {
    observed_at: i64,
    evaluated_at: i64,
    values: [Option<u8>; 3],
    limits: [u8; 3],
}
impl CapacityRefusal {
    pub(crate) fn new(
        observed_at: i64,
        evaluated_at: i64,
        values: [Option<u8>; 3],
        limits: [u8; 3],
    ) -> Option<Self> {
        let proof = Self {
            observed_at,
            evaluated_at,
            values,
            limits,
        };
        (observed_at > 0
            && evaluated_at >= observed_at
            && values.into_iter().flatten().all(|v| v <= 100)
            && limits.into_iter().all(|v| v <= 100)
            && proof.crossed().into_iter().any(|v| v))
        .then_some(proof)
    }
    fn crossed(&self) -> [bool; 3] {
        [
            self.values[0].is_some_and(|v| v >= self.limits[0]),
            self.values[1].is_some_and(|v| v <= self.limits[1]),
            self.values[2].is_some_and(|v| v >= self.limits[2]),
        ]
    }
    pub(crate) fn encoded(&self) -> String {
        format!(
            "resource_saturated:{}",
            json!([
                self.observed_at,
                self.evaluated_at,
                self.values,
                self.limits
            ])
        )
    }
    pub(crate) fn decode(code: &str) -> Option<Self> {
        if code.len() > 128 {
            return None;
        }
        let body = code.strip_prefix("resource_saturated:")?;
        let (observed, evaluated, values, limits) =
            serde_json::from_str::<(i64, i64, [Option<u8>; 3], [u8; 3])>(body).ok()?;
        Self::new(observed, evaluated, values, limits)
    }
    pub(crate) fn projection(&self) -> Value {
        let crossed = self
            .crossed()
            .into_iter()
            .zip(["cpu", "available_memory", "normalized_load"])
            .filter_map(|(yes, name)| yes.then_some(name))
            .collect::<Vec<_>>();
        json!({"evidence_available":true,"scope":"historical_refusal","observed_at":self.observed_at,
            "evaluated_at":self.evaluated_at,"machine":"imposed",
            "measurements":{"cpu_percent":self.values[0],"available_memory_percent":self.values[1],"normalized_load_percent":self.values[2]},
            "thresholds":{"cpu_saturation_percent":self.limits[0],"memory_available_min_percent":self.limits[1],"normalized_load_saturation_percent":self.limits[2]},
            "crossed":crossed})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec189_capacity_proof_roundtrip_and_closed_fields() {
        let proof =
            CapacityRefusal::new(100, 105, [Some(35), Some(9), Some(28)], [90, 10, 100]).unwrap();
        assert!(proof.encoded().len() <= 128);
        assert_eq!(
            CapacityRefusal::decode(&proof.encoded()),
            Some(proof.clone())
        );
        assert_eq!(proof.projection()["crossed"], json!(["available_memory"]));
        assert_eq!(proof.projection()["scope"], "historical_refusal");
        assert!(CapacityRefusal::decode("resource_saturated:{\"secret\":\"private\"}").is_none());
    }
    #[test]
    fn spec189_missing_measurement_is_unknown_not_zero() {
        let proof = CapacityRefusal::new(100, 100, [None, Some(10), None], [90, 10, 100]).unwrap();
        assert_eq!(
            proof.projection()["measurements"]["cpu_percent"],
            Value::Null
        );
        assert!(CapacityRefusal::new(100, 100, [None, None, None], [90, 10, 100]).is_none());
        assert!(
            CapacityRefusal::new(100, 100, [Some(35), Some(11), Some(28)], [90, 10, 100]).is_none()
        );
        assert!(CapacityRefusal::new(101, 100, [Some(100), None, None], [90, 10, 100]).is_none());
        assert!(CapacityRefusal::new(100, 100, [Some(255), None, None], [90, 10, 100]).is_none());
        assert!(CapacityRefusal::decode(&"x".repeat(129)).is_none());
    }
}
