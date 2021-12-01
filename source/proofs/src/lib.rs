mod merkle;
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use std::vec;

// use super::utils;
use bellman::gadgets::boolean::{AllocatedBit, Boolean};
use bellman::groth16::Parameters;
use bellman::{groth16, Circuit, ConstraintSystem, SynthesisError};
use bls12_381::{Bls12, Scalar};
use ff::Field;
use ff::PrimeField;
use rand::rngs::OsRng;
use rand::thread_rng;

use crate::merkle::Tree;


const TREE_DEPTH: usize = 3;
pub const MIMC_ROUNDS: usize = 322;

struct MyCircuit<'a, S: PrimeField> {
    // rt: [u8; 32],
    // leaf: Option<[u8; 32]>,
    leaf: Option<S>,
    // auth_path: [Option<([u8; 32], bool)>; TREE_DEPTH],
    auth_path: [Option<(S, bool)>; TREE_DEPTH],
    constants: &'a [S],
}

fn convert_to_str(v: Vec<Boolean>) -> String {
    let mut str = String::from("");
    for (idx, elem) in v.iter().enumerate() {
        if let Some(e) = elem.get_value() {
            if e {
                str.push('1');
            } else {
                str.push('0');
            }
        } else {
            str.push('_');
        }

        if idx % 8 == 7 {
            str.push(' ');
        }
    }

    str
}

impl<'a, S: PrimeField> Circuit<S> for MyCircuit<'a, S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        println!("\nsynthesize()");
        let mut cur = match self.leaf {
            Some(a) => Some(a),
            None => Some(S::default()),
        };
        {
            for (idx, layer) in self.auth_path.into_iter().enumerate() {
                let cs = &mut cs.namespace(|| format!("layer {}", idx));

                let cur_is_right = AllocatedBit::alloc(
                    cs.namespace(|| "cur is right"),
                    layer.as_ref().map(|&(_, d)| d),
                )
                .unwrap();

                println!(
                    "\nlayer_idx: {}, curr_is_right: {:?}",
                    idx,
                    cur_is_right.get_value()
                );

                // start mimc
                let mut xl_value;
                let mut xr_value;

                // let right = cur_is_right.get_value().unwrap();
                let right =
                    cur_is_right
                        .get_value()
                        .and_then(|v| if v { Some(v) } else { Some(false) });

                let temp = match *layer {
                    Some(a) => a,
                    None => (S::default(), false),
                };

                if match right {
                    Some(a) => a,
                    None => false,
                } {
                    xl_value = Some(temp.0);
                    xr_value = cur;
                } else {
                    xl_value = cur;
                    xr_value = Some(temp.0);
                }

                let mut xl = cs.alloc(
                    || "preimage xl",
                    || xl_value.ok_or(SynthesisError::AssignmentMissing),
                )?;

                // Allocate the second component of the preimage.
                // let mut xr_value = self.xr;
                let mut xr = cs.alloc(
                    || "preimage xr",
                    || xr_value.ok_or(SynthesisError::AssignmentMissing),
                )?;

                for i in 0..MIMC_ROUNDS {
                    // xL, xR := xR + (xL + Ci)^3, xL
                    // let cs = &mut cs.namespace(|| format!("round {}", i));

                    // tmp = (xL + Ci)^2
                    let tmp_value = xl_value.map(|mut e| {
                        e.add_assign(&self.constants[i]);
                        e.square()
                    });
                    let tmp = cs.alloc(
                        || "tmp",
                        || tmp_value.ok_or(SynthesisError::AssignmentMissing),
                    )?;

                    cs.enforce(
                        || "tmp = (xL + Ci)^2",
                        |lc| lc + xl + (self.constants[i], CS::one()),
                        |lc| lc + xl + (self.constants[i], CS::one()),
                        |lc| lc + tmp,
                    );

                    // new_xL = xR + (xL + Ci)^3
                    // new_xL = xR + tmp * (xL + Ci)
                    // new_xL - xR = tmp * (xL + Ci)
                    let new_xl_value = xl_value.map(|mut e| {
                        e.add_assign(&self.constants[i]);
                        e.mul_assign(&tmp_value.unwrap());
                        e.add_assign(&xr_value.unwrap());
                        e
                    });

                    let new_xl = cs.alloc(
                        || "new_xl",
                        || new_xl_value.ok_or(SynthesisError::AssignmentMissing),
                    )?;

                    cs.enforce(
                        || "new_xL = xR + (xL + Ci)^3",
                        |lc| lc + tmp,
                        |lc| lc + xl + (self.constants[i], CS::one()),
                        |lc| lc + new_xl - xr,
                    );

                    // xR = xL
                    xr = xl;
                    xr_value = xl_value;

                    // xL = new_xL
                    xl = new_xl;
                    xl_value = new_xl_value;
                }

                cur = xl_value;
                println!("circuit public input {:?}", cur.unwrap());
                // end of mimc

                // let cur_str = convert_to_str(cur.clone());
                // println!("\nlayer_idx: {}, cur: {}", idx, cur_str);
            }
        };
        cs.alloc_input(|| "image", || cur.ok_or(SynthesisError::AssignmentMissing))?;
        println!("final circuit public input {:?}", cur.unwrap());

        Ok(())
    }
}

pub fn mimc<S: PrimeField>(mut xl: S, mut xr: S, constants: &[S]) -> S {
    assert_eq!(constants.len(), MIMC_ROUNDS);

    for c in constants {
        let mut tmp1 = xl;
        tmp1.add_assign(c);
        let mut tmp2 = tmp1.square();
        tmp2.mul_assign(&tmp1);
        tmp2.add_assign(&xr);
        xr = xl;
        xl = tmp2;
    }

    xl
}

pub fn mimcTest() {
    println!("start");
    // let test_leaves: Vec<u32> = (0..std::u32::MAX).map(|x| x).collect();
    let mut test_leaves: Vec<u32> = vec![];
    (0..2).for_each(|iter| {
        test_leaves.push(iter.clone());
    });
    println!("before new tree");

    let mut rng = thread_rng();

    let constants = (0..MIMC_ROUNDS)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();

    let tree = Tree::new(test_leaves, TREE_DEPTH, &constants);

    println!("before generate proof");
    let proof = tree.generate_proof(1);
    let leaf = tree.nodes.get(0).unwrap().get(1).unwrap().hash.to_bytes();
    let root = tree.root().hash;

    println!("\nproof: {:?}", proof);
    println!("\nroot: {:?}", root);

    let now = SystemTime::now();
    println!("timer start {:?}", now);
    let params = {
        let c = MyCircuit {
            leaf: None,
            auth_path: [None; TREE_DEPTH],
            constants: &constants,
        };

        groth16::generate_random_parameters::<Bls12, _, _>(c, &mut OsRng).unwrap()
    };

    let param_time = SystemTime::now();
    println!(
        "generate random parameter end {:?}",
        param_time.duration_since(now)
    );

    // write param to file
    let mut file = File::create("new_params").unwrap();
    let mut v = vec![];

    params.write(&mut v).unwrap();

    println!("writed data len: {}", v.len());

    // write origin buf
    file.write_all(&v);

    // let strings: Vec<String> = v.iter().map(|n| n.to_string()).collect();
    // write!(file, "{}", strings.join(",")).unwrap();

    // read param from file
    // let contents = std::fs::read_to_string("params").unwrap();
    // let v2 = std::fs::read("params_8").unwrap();

    // let v2: Vec<u8> = contents
    //     .split(",")
    //     .map(|x| x.parse::<u8>().unwrap())
    //     .collect();

    println!("parsed data len: {}", v.len());

    let de_params = Parameters::<Bls12>::read(&v[..], false).unwrap();

    // Prepare the verification key (for proof verification).
    let pvk = groth16::prepare_verifying_key(&de_params.vk);

    let verify_key_time = SystemTime::now();
    println!(
        "prepare verify key {:?}",
        verify_key_time.duration_since(param_time)
    );

    // Create an instance of our circuit (with the preimage as a witness).
    let mut auth_path: [Option<(Scalar, bool)>; TREE_DEPTH] = [None; TREE_DEPTH];
    for (idx, elem) in auth_path.clone().iter().enumerate() {
        let sib = proof.get(idx).unwrap();
        auth_path[idx] = Some((sib.hash.clone(), sib.direction.clone()));
    }
    let auth_path_time = SystemTime::now();
    println!(
        "auth_path generate time {:?}",
        auth_path_time.duration_since(verify_key_time)
    );

    let c = MyCircuit {
        leaf: Some(Scalar::from_bytes(&leaf).unwrap()),
        auth_path: auth_path,
        constants: &constants,
    };
    let circuit_time = SystemTime::now();
    println!(
        "circuit time {:?}",
        circuit_time.duration_since(auth_path_time)
    );

    // let root_bits = multipack::bytes_to_bits_le(&root);
    // let inputs = multipack::compute_multipacking(&root_bits);
    let multipacking_time = SystemTime::now();
    println!(
        "multipacking time {:?}",
        multipacking_time.duration_since(circuit_time)
    );

    let proof = groth16::create_random_proof(c, &de_params, &mut OsRng).unwrap();
    let proof_time = SystemTime::now();
    println!(
        "proof time {:?}",
        proof_time.duration_since(multipacking_time)
    );

    match groth16::verify_proof(&pvk, &proof, &[root]) {
        Ok(_) => (println!("veryfiy success!")),
        Err(err) => {
            println!("verify_proof(), err: {}", err);
        }
    }

    let verify_finish_time = SystemTime::now();
    println!(
        "verify proof finished {:?}",
        verify_finish_time.duration_since(proof_time)
    );
}