use rug::{Integer, Assign};
use rug::rand::RandState;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Serialize, Deserialize};

/// Params for pailliar key
#[derive(Debug, Serialize, Deserialize)]
pub struct PailliarPublicKey {
    pub n: Integer,
    pub n_sq: Integer,
    pub g: Integer
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PailliarPrivateKeyShare{
    /// Share value part of lambda
    pub share: Integer,
    /// Share index (0..=n)
    pub index: i32,
}

pub struct PailliarPrivateKey{
    pub lambda: Integer,
    pub mu: Integer
}