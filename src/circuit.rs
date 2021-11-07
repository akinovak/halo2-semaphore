use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, Region, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance, Selector},
    poly::Rotation,
};

// Semaphore config
#[derive(Clone, Debug)]
pub struct Config {
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
        Config {}
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // Return empty for now
        { Ok({}) }
    }

}
