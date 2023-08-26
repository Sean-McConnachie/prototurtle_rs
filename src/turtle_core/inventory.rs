use super::data::TurtSlot;
use super::control::TurtControl;

pub const TURT_SLOTS: usize = 16;

#[derive(Debug, Clone)]
pub enum TurtBlock {
    None,
    Any,
    Some(String),
}

#[derive(Debug)]
pub struct TurtInventory<'a> {
    turt: &'a TurtControl<'a>,
    slots: Vec<Option<TurtSlot>>,
}

impl<'a> TurtInventory<'a> {
    pub fn init(turt: &'a TurtControl<'a>) -> Self {
        Self {
            turt,
            slots: vec![None; TURT_SLOTS],
        }
    }

    pub fn full_update(&mut self) {
        for s in 0..TURT_SLOTS {
            self.slots[s] = self.turt.inv_item_detail(s as u8)
        }
    }

    pub fn is_full(&self) -> bool {
        for s in self.slots.iter() {
            if s.is_none() {
                return false;
            }
        }
        true
    }
}
