use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner, Region},
    plonk::{Advice, Instance, Circuit, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
    pasta::Fp
};

mod gadget;
mod utils;

use gadget:: {
    add::{AddChip, AddConfig, AddInstruction}
};

use crate:: {
    utils::{UtilitiesInstructions, CellValue}
};

// Absolute offsets for public inputs.
const IDENTITY_COMMITMENT: usize = 0;
const NULLIFIER_HASH: usize = 1;

// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
    advices: [Column<Advice>; 3],
    instance: Column<Instance>,
    add_config: AddConfig,
    s_external: Selector,
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit<F> {
    identity_trapdoor: Option<F>,
    identity_nullifier: Option<F>,
    external_nullifier: Option<F>,
}

impl<F: FieldExt> UtilitiesInstructions<F> for SemaphoreCircuit<F> {
    type Var = CellValue<F>;
}

impl<F: FieldExt> Circuit<F> for SemaphoreCircuit<F> {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {

        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        for advice in advices.iter() {
            meta.enable_equality((*advice).into());
        }

        // [0..2].try_into().unwrap()
        let add_config = AddChip::configure(meta, advices[0..2].try_into().unwrap());

        let s_external = meta.selector();
        meta.create_gate("external nullifier", |meta| {
            let advice_input = meta.query_advice(advices[2], Rotation::cur());
            let public_input = meta.query_instance(instance, Rotation::cur());
            let s_external = meta.query_selector(s_external);
            vec![s_external * (advice_input - public_input)]
        });

        Config {
            advices, 
            instance,
            add_config,
            s_external
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {

        let add_chip = AddChip::<F>::construct(config.add_config);
        
        let identity_trapdoor = self.load_private(
            layouter.namespace(|| "witness identity_trapdoor"),
            config.advices[0],
            self.identity_trapdoor,
        );

        let identity_nullifier = self.load_private(
            layouter.namespace(|| "witness identity_nullifier"),
            config.advices[0],
            self.identity_nullifier,
        );

        layouter.assign_region(
            || "external nullifier",
            |mut region: Region<'_, F>| {

                let a = region.assign_advice(
                    || "external",
                    config.advices[2],
                    0,
                    || self.external_nullifier.ok_or(Error::SynthesisError),
                )?;
                // region.constrain_equal(a.cell, lhs)?;
                config.s_external.enable(&mut region, 0)?;
                Ok(())
            },
        )?;


        let identity_commitment = add_chip.add(layouter.namespace(|| "commitment"), identity_nullifier.unwrap(), identity_trapdoor.unwrap())?;
        // let nullifier_hash = add_chip.add(layouter.namespace(|| "nullifier"), identity_nullifier.unwrap(), external_nullifier.unwrap())?;


        // TODO merkle chip for membership proof
        // TODO calc nullifier hash = hash(identity_nullifier, external_nullifier)
        // let nullifier_hash = add_chip.add(layouter.namespace(|| "nullifier"), identity_nullifier.unwrap(), identity_trapdoor.unwrap())?;
        self.constrain_public(layouter.namespace(|| "constrain identity_commitment"), config.instance, identity_commitment, IDENTITY_COMMITMENT);
        // self.constrain_public(layouter.namespace(|| "constrain nullifier_hash"), config.instance, nullifier_hash, NULLIFIER_HASH);

        Ok({})
    }

}


fn main() {
    use halo2::{dev::MockProver};

    let k = 4;

    let identity_trapdoor = Fp::from(2);
    let identity_nullifier = Fp::from(3);
    let external_nullifier = Fp::from(5);
    let identity_commitment = identity_trapdoor + identity_nullifier;

    let circuit = SemaphoreCircuit {
        identity_trapdoor: Some(identity_trapdoor),
        identity_nullifier: Some(identity_nullifier),
        external_nullifier: Some(external_nullifier),
    };

    let external_nullifier = Fp::from(10);
    let mut public_inputs = vec![identity_commitment, external_nullifier + external_nullifier];

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    public_inputs[0] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_err());
}
