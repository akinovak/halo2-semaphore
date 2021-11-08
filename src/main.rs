use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Instance, Circuit, Column, ConstraintSystem, Error},
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


// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
    advices: [Column<Advice>; 2],
    instance: Column<Instance>,
    add_config: AddConfig
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit<F> {
    identity_trapdoor: Option<F>,
    identity_nullifier: Option<F>,
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
        ];

        let instance = meta.instance_column();
        meta.enable_equality(instance.into());

        let add_config = AddChip::configure(meta, advices);

        Config {
            advices, 
            instance,
            add_config
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

        let identity_commitment = add_chip.add(layouter.namespace(|| "a + b"), identity_nullifier.unwrap(), identity_trapdoor.unwrap())?;
        self.expose_public(layouter.namespace(|| "expose identity_commitment"), config.instance, identity_commitment, 0);
        
        // TODO merkle chip for membership proof
        // TODO calc nullifier hash = hash(identity_nullifier, external_nullifier)

        Ok({})
    }

}


fn main() {
    use halo2::{dev::MockProver};

    let k = 4;

    let identity_trapdoor = Fp::from(2);
    let identity_nullifier = Fp::from(3);
    let identity_commitment = identity_trapdoor + identity_nullifier;

    let circuit = SemaphoreCircuit {
        identity_trapdoor: Some(identity_trapdoor),
        identity_nullifier: Some(identity_nullifier),
    };

    let mut public_inputs = vec![identity_commitment];

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    public_inputs[0] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_err());
}
