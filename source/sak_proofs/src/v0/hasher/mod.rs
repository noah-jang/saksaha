use crate::{get_mimc_constants, mimc, mimc_cs, ProofError};
use bellman::{ConstraintSystem, SynthesisError};
use bls12_381::Scalar;
use ff::PrimeField;

pub struct Hasher {
    constants: Vec<Scalar>,
}

impl Hasher {
    #[allow(dead_code)]
    pub fn new() -> Hasher {
        let mimc_constants = get_mimc_constants();

        Hasher {
            constants: mimc_constants,
        }
    }

    pub fn mimc2(
        &self,
        xl: &[u8; 32],
        xr: &[u8; 32],
    ) -> Result<[u8; 32], ProofError> {
        let xl = {
            let ct_option = Scalar::from_bytes(xl);

            if bool::from(ct_option.is_some()) {
                ct_option.unwrap()
            } else {
                return Err(format!("Convert to scalar has failed").into());
            }
        };

        let xr = {
            let ct_option = Scalar::from_bytes(xr);
            if bool::from(ct_option.is_some()) {
                ct_option.unwrap()
            } else {
                return Err(format!("Convert to scalar has failed").into());
            }
        };

        let res = mimc(xl, xr, &self.constants);

        Ok(res.to_bytes())
    }

    #[allow(dead_code)]
    fn mimc_cs_scalar<CS: ConstraintSystem<Scalar>>(
        &self,
        cs: &mut CS,
        a: Option<Scalar>,
        b: Option<Scalar>,
    ) -> Option<Scalar> {
        mimc_cs(cs, a, b, &self.constants)
    }

    /// pseudo random function
    #[allow(dead_code)]
    pub fn prf(&self, z: Scalar, x: Scalar) -> Scalar {
        mimc(z, x, &self.constants)
    }

    /// pseudo random function for constraint system
    #[allow(dead_code)]
    pub fn prf_cs<CS: ConstraintSystem<Scalar>>(
        &self,
        cs: &mut CS,
        z: Option<Scalar>,
        x: Option<Scalar>,
    ) -> Option<Scalar> {
        self.mimc_cs_scalar(cs, z, x)
    }

    pub fn comm2(&self, r: &[u8; 32], x: &[u8; 32]) -> &[u8; 32] {
        mimc(r, x, &self.constants)
    }

    /// commitment generating hash function
    #[allow(dead_code)]
    pub fn comm(&self, r: Scalar, x: Scalar) -> Scalar {
        mimc(r, x, &self.constants)
    }

    /// commitment generating hash function for constraint system
    #[allow(dead_code)]
    pub fn comm_cs<CS: ConstraintSystem<Scalar>>(
        &self,
        cs: &mut CS,
        r: Option<Scalar>,
        x: Option<Scalar>,
    ) -> Option<Scalar> {
        self.mimc_cs_scalar(cs, r, x)
    }
}
