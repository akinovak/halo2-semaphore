use halo2::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::{Error}
};

mod chip;
pub use chip::{MerkleConfig, MerkleChip};
use super::super::MERKLE_DEPTH;


pub trait MerkleInstructions<F: FieldExt> 
: Chip<F> 
{
    type Cell;

    fn hash_layer(
        &self,
        layouter: impl Layouter<F>,
        leaf_or_digest: Self::Cell,
        sibling: Option<F>,
        position_bit: Option<F>,
        layer: usize,
    ) -> Result<Self::Cell, Error>;

}

#[derive(Clone, Debug)]
pub struct MerklePath<F: FieldExt, MerkleChip> 
where MerkleChip: MerkleInstructions<F> + Clone,
{
    pub chip: MerkleChip,
    pub leaf_pos: Option<[F; MERKLE_DEPTH]>,
    // The Merkle path is ordered from leaves to root.
    pub path: Option<[F; MERKLE_DEPTH]>,
}

impl<F: FieldExt> 
    MerklePath<F, MerkleChip<F>,
    > where MerkleChip<F> : MerkleInstructions<F> + Clone,
    {
    pub fn calculate_root(
        &self,
        mut layouter: impl Layouter<F>,
        leaf: <MerkleChip<F> as MerkleInstructions<F>>::Cell,
    ) -> Result<<MerkleChip<F> as MerkleInstructions<F>>::Cell, Error> {
        let mut node = leaf;
        
        let path = self.path.unwrap();
        let leaf_pos = self.leaf_pos.unwrap(); 

        for (layer, (sibling, pos)) in path.iter().zip(leaf_pos.iter()).enumerate() {
            // println!("usao: {:?}", sibling);
            node = self.chip.hash_layer(layouter.namespace(|| format!("hash l {}", layer)), node, Some(*sibling), Some(*pos), layer)?;
        }

        Ok(node)
    }
}