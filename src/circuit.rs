use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, Region, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance, Selector},
    poly::Rotation,
};

use gadget:: {
    add:: {
        chip::{AddConfig, AddChip},
    }
};

// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
    advices: [Column<Advice>; 2],
    add_config: AddConfig
}

// Semaphore circuit
#[derive(Debug, Default)]
pub struct SemaphoreCircuit {
}

impl<F: FieldExt> Circuit<F> for SemaphoreCircuit {
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
    }

}
