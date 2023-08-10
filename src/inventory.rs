use crate::turtle;

#[derive(Debug, Clone)]
pub struct Inventory<'a> {
    turt: &'a turtle::Turt<'a>,
    slots: Vec<Option<turtle::TurtSlot>>,
}

impl<'a> Inventory<'a> {
    pub fn init(turt: &'a turtle::Turt<'a>) -> Self {
        Self {
            turt,
            slots: vec![None; 16],
        }
    }

    pub fn full_update(&mut self) {
        for s in 0..16 {
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

