mod coin_proof;
mod hasher;
mod merkle;
mod mimc;

#[cfg(test)]
mod tests;

pub use coin_proof::*;
pub use hasher::*;
pub use merkle::*;

pub use bellman::gadgets::boolean::AllocatedBit;
pub use bellman::groth16::{
    Parameters, PreparedVerifyingKey, Proof,
    VerifyingKey as Groth16VerifyingKey,
};
pub use bellman::{groth16, Circuit, ConstraintSystem, SynthesisError};
pub use bls12_381::{Bls12, Scalar};

pub type ProofError = Box<dyn std::error::Error + Send + Sync>;