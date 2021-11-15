use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Error},
    pasta::Fp
};

mod chip;
pub use chip::{MerkleConfig, MerkleChip};
use super::super::MERKLE_DEPTH;


pub trait MerkleInstructions
: Chip<Fp> 
{
    type Cell;

    fn hash_layer(
        &self,
        layouter: impl Layouter<Fp>,
        leaf_or_digest: Self::Cell,
        sibling: Option<Fp>,
        position_bit: Option<Fp>,
        layer: usize,
    ) -> Result<Self::Cell, Error>;

}

#[derive(Clone, Debug)]
pub struct MerklePath<MerkleChip> 
where MerkleChip: MerkleInstructions + Clone,
{
    pub chip: MerkleChip,
    pub leaf_pos: Option<[Fp; MERKLE_DEPTH]>,
    // The Merkle path is ordered from leaves to root.
    pub path: Option<[Fp; MERKLE_DEPTH]>,
}

impl MerklePath<MerkleChip,
    > where MerkleChip : MerkleInstructions + Clone,
    {
    pub fn calculate_root(
        &self,
        mut layouter: impl Layouter<Fp>,
        leaf: <MerkleChip as MerkleInstructions>::Cell,
    ) -> Result<<MerkleChip as MerkleInstructions>::Cell, Error> {
        let mut node = leaf;
        
        let path = self.path.unwrap();
        let leaf_pos = self.leaf_pos.unwrap(); 

        for (layer, (sibling, pos)) in path.iter().zip(leaf_pos.iter()).enumerate() {
            node = self.chip.hash_layer(layouter.namespace(|| format!("hash l {}", layer)), node, Some(*sibling), Some(*pos), layer)?;
        }

        Ok(node)
    }
}