use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
    pasta::Fp
};

mod gadget;
mod utils;

use gadget:: {
    add::{AddChip, AddConfig}
};

use crate:: {
    utils::{UtilitiesInstructions, CellValue}
};


// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
    advices: [Column<Advice>; 2],
    add_config: AddConfig
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit<F> {
    a: Option<F>,
    b: Option<F>,
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

        let add_config = AddChip::configure(meta, advices);

        Config {
            advices, 
            add_config
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // Return empty for now
        Ok({})
        // let psi_old = self.load_private(
        //     layouter.namespace(|| "witness psi_old"),
        //     config.advices[0],
        //     Fp::from(3)
        // )?;
    }

}


fn main() {
    let a = Fp::from(2);
    let b = Fp::from(3);

    let semaphore_circuit = SemaphoreCircuit {
        a: Some(a),
        b: Some(b),
    };
}
