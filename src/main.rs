use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner, Region},
    plonk::{Advice, Instance, Circuit, Column, ConstraintSystem, Error, Selector},
    pasta::Fp
};
use std::marker::PhantomData;

mod gadget;
mod utils;

use gadget:: {
    add::{AddChip, AddConfig, AddInstruction},
    merkle::{MerkleChip, MerkleConfig, MerklePath}
};

use crate:: {
    utils::{UtilitiesInstructions, CellValue, Var}
};

pub const MERKLE_DEPTH: usize = 1;

// Absolute offsets for public inputs.
const EXTERNAL_NULLIFIER: usize = 0;
const NULLIFIER_HASH: usize = 1;

// Semaphore config
#[derive(Clone, Debug)]
pub struct Config<F> {
    advices: [Column<Advice>; 4],
    instance: Column<Instance>,
    add_config: AddConfig,
    merkle_config: MerkleConfig,
    s_external: Selector,
    pub _marker: PhantomData<F>,
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit<F> {
    identity_trapdoor: Option<F>,
    identity_nullifier: Option<F>,
    external_nullifier: Option<F>,
    position_bits: Option<[F; MERKLE_DEPTH]>,
    path: Option<[F; MERKLE_DEPTH]>
}

impl<F: FieldExt> UtilitiesInstructions<F> for SemaphoreCircuit<F> {
    type Var = CellValue<F>;
}

impl<F: FieldExt> Circuit<F> for SemaphoreCircuit<F> {
    type Config = Config<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {

        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        for advice in advices.iter() {
            meta.enable_equality((*advice).into());
        }

        let add_config = AddChip::configure(meta, advices[0..2].try_into().unwrap());
        let merkle_config = MerkleChip::configure(meta, advices[0..3].try_into().unwrap());

        let s_external = meta.selector();
        // TODO check why this is not working
        // meta.create_gate("external nullifier", |meta| {
        //     let s_external = meta.query_selector(s_external);
        //     let advice_input = meta.query_advice(advices[2], Rotation::cur());
        //     let public_input = meta.query_instance(instance, Rotation::cur());

        //     // println!("In configure: {:?}", public_input);

        //     vec![s_external * (advice_input - public_input)]
        // });

        Config {
            advices, 
            instance,
            add_config,
            merkle_config,
            s_external,
            _marker: PhantomData
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {

        let add_chip = config.construct_add_chip();
        let merkle_chip = config.construct_merkle_chip();
        
        let identity_trapdoor = self.load_private(
            layouter.namespace(|| "witness identity_trapdoor"),
            config.advices[0],
            self.identity_trapdoor,
        )?;

        let identity_nullifier = self.load_private(
            layouter.namespace(|| "witness identity_nullifier"),
            config.advices[0],
            self.identity_nullifier,
        )?;

        let external_nulifier_cell = layouter.assign_region(
            || "external nullifier",
            |mut region: Region<'_, F>| {

                config.s_external.enable(&mut region, 0)?;

                let cell = region.assign_advice(
                    || "external",
                    config.advices[2],
                    0,
                    || self.external_nullifier.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, self.external_nullifier))
            },
        )?;

        let identity_commitment = add_chip.add(layouter.namespace(|| "commitment"), identity_nullifier, identity_trapdoor)?;
        let nullifier_hash = add_chip.add(layouter.namespace(|| "nullifier"), identity_nullifier, external_nulifier_cell)?;

        // assert_eq!(self.position_bits.len(), MERKLE_DEPTH);

        let merkle_inputs = MerklePath {
            chip: merkle_chip,
            leaf_pos: self.position_bits,
            path: self.path
        };

        // for i in 0..MERKLE_DEPTH {
        //     merkle_chip.hash_layer(layouter.namespace(|| "merkle namespace"), self.path_bit[i], self.identity_trapdoor, self.identity_nullifier, i)?;

        // }

        // self.constrain_public(layouter.namespace(|| "constrain identity_commitment"), config.instance, identity_commitment, IDENTITY_COMMITMENT);
        self.constrain_public(layouter.namespace(|| "constrain external_nullifier"), config.instance, external_nulifier_cell, EXTERNAL_NULLIFIER)?;
        self.constrain_public(layouter.namespace(|| "constrain nullifier_hash"), config.instance, nullifier_hash, NULLIFIER_HASH)?;

        Ok({})
    }
}


fn main() {
    use halo2::{dev::MockProver};

    let k = 4;

    let identity_trapdoor = Fp::from(2);
    let identity_nullifier = Fp::from(3);
    let external_nullifier = Fp::from(5);
    let path = Fp::from(10);
    let position_bits = Fp::from(0);
    let identity_commitment = identity_trapdoor + identity_nullifier;
    let nullifier_hash = identity_nullifier + external_nullifier;

    let circuit = SemaphoreCircuit {
        identity_trapdoor: Some(identity_trapdoor),
        identity_nullifier: Some(identity_nullifier),
        external_nullifier: Some(external_nullifier),
        position_bits: Some([position_bits]),
        path: Some([path])
    };

    let mut public_inputs = vec![external_nullifier, nullifier_hash];

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    // public_inputs[0] += Fp::one();
    // let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    // assert!(prover.verify().is_err());
}
