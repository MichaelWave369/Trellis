use serde::{Deserialize, Serialize};

pub mod checksum;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumState {
    Verified,
    Unverified,
    Unavailable,
    Mismatched,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureState {
    Present,
    Missing,
    Malformed,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAssessment {
    pub state: SignatureState,
    pub value: Option<String>,
    pub note: String,
}

pub fn assess_signature(signature: Option<&str>) -> SignatureAssessment {
    match signature.map(str::trim) {
        None | Some("") => SignatureAssessment {
            state: SignatureState::Missing,
            value: None,
            note: "no signature metadata declared".to_string(),
        },
        Some(raw) if raw.starts_with("sig:") && raw.len() > 8 => SignatureAssessment {
            state: SignatureState::Present,
            value: Some(raw.to_string()),
            note:
                "signature metadata present; cryptographic verification is not implemented in v0.4"
                    .to_string(),
        },
        Some(raw) => SignatureAssessment {
            state: SignatureState::Malformed,
            value: Some(raw.to_string()),
            note: "signature metadata is malformed; expected placeholder format sig:<value>"
                .to_string(),
        },
    }
}
