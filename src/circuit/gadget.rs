use add::AddChip;

pub(crate) mod add;

impl super::Config {
    pub(super) fn add_chip(&self) -> AddChip {
        AddChip::construct(self.add_config.clone())
    }
}