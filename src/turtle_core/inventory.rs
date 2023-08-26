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
    pub slots: Vec<Option<TurtSlot>>,
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

    pub fn reduce_count_andor_find_next(&mut self, start_slot: usize) -> Option<i32> {
        let mut slot = start_slot;
        let mut ignore = false;
        'redo: loop {
            if let Some(s) = &mut self.slots[slot] {
                if s.count() > 0 {
                    s.reduce_count(1);
                    return Some(s.count());
                } else if s.count() == 0 {
                    if !ignore {
                        self.slots[slot] = None;
                        ignore = true;
                        continue 'redo;
                    }
                };
            } else {
                if !ignore {
                    ignore = true;
                    self.slots[slot] = self.turt.inv_item_detail(slot as u8);
                    continue 'redo;
                }
            };
            ignore = false;
            slot += 1;
            if slot >= TURT_SLOTS {
                slot = 0;
            }
            if slot == start_slot {
                return None;
            }
        }
    }
}
