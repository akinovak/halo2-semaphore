use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::{Error}
};

mod chip;
pub use chip::{MerkleConfig, MerkleChip};


pub trait MerkleInstructions<F: FieldExt> 
: Chip<F> 
{
    type Cell;

    fn check_bool(
        &self,
        layouter: impl Layouter<F>,
        position_bit: Self::Cell,
        layer: usize,
    ) -> Result<(), Error>;

}