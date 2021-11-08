use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::{Error}
};

mod chip;
pub use chip::{AddConfig, AddChip};

pub trait AddInstruction<F: FieldExt> 
: Chip<F> 
{
    type Cell;

    fn add(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Cell,
        b: Self::Cell,
    ) -> Result<Self::Cell, Error>;

}