use crate::{DefaultData, PROGRESS_DIR, TurtleIdentifier};
use crate::turtle_core::control::TurtControl;
use crate::turtle_core::navigation::{Head, Pos, PosH, TurtNavigation};
use crate::turtle_core::inventory::{TurtInventory, TURT_SLOTS, TurtBlock};
use modelutils_rs::coords::Order;

use std::path;
use std::path::PathBuf;
use crate::turtle_core::file_system_storage::{FStore, fstore_load_or_init, fstore_save};


#[derive(Debug, Clone)]
pub struct ChunkDiggerConfig {
    pub p1: Pos,
    pub p2: Pos,
    pub place_floor: TurtBlock,
    pub chest_size: usize,
    pub check_inv_every_n_blocks: usize,
}

#[derive(Debug)]
struct FStoreChunkDigger {
    fp: PathBuf,
    layer: usize,
    stack_count: usize,
}

impl FStore for FStoreChunkDigger {
    fn default(p: &PathBuf) -> Self {
        Self {
            fp: p.clone(),
            layer: 0,
            stack_count: 0,
        }
    }

    fn path(&self) -> &PathBuf {
        &self.fp
    }

    fn save(&self) -> String {
        format!("{}\n{}\n", self.layer, self.stack_count)
    }

    fn load(p: &PathBuf, d: &str) -> Self {
        let lines: Vec<String> = d.lines().map(String::from).collect();
        Self {
            fp: p.clone(),
            layer: lines[0].parse::<usize>().unwrap(),
            stack_count: lines[1].parse::<usize>().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct ChunkDigger<'a> {
    _identifier: TurtleIdentifier,
    _index: usize,
    turt: &'a TurtControl<'a>,
    nav: &'a mut TurtNavigation<'a>,
    inv: TurtInventory<'a>,

    conf: ChunkDiggerConfig,
    fstore_chunk_digger: FStoreChunkDigger,
}

impl<'a> ChunkDigger<'a> {
    pub fn init(data: DefaultData<'a>, conf: ChunkDiggerConfig) -> Self {
        let fp = path::PathBuf::from(
            format!("{}/{}.chunkdigger", PROGRESS_DIR, data.0));
        let fstore_chunk_digger = fstore_load_or_init::<FStoreChunkDigger>(&fp);
        Self {
            _identifier: data.0,
            _index: data.1,
            turt: data.2,
            nav: data.3,
            inv: TurtInventory::init(&data.2),
            conf,
            fstore_chunk_digger,
        }
    }

    fn inv_check(&mut self) {
        self.inv.full_update();
        if self.inv.is_full() {
            let chest_size = self.conf.chest_size;

            // Increment number of slots mined
            self.fstore_chunk_digger.stack_count += TURT_SLOTS;
            // Save position of turtle (to return to)
            let saved_pos = self.nav.pos().clone();
            // Calculate offset from starting position
            let offset = (self.fstore_chunk_digger.stack_count - TURT_SLOTS) / chest_size;
            // Calculate number of slots to place in chest
            let max_chest_space = chest_size - ((self.fstore_chunk_digger.stack_count - TURT_SLOTS) % chest_size);

            // Calculate first chest location and go there
            let mut chest_loc: PosH = self.conf.p1.clone().into();
            chest_loc.z = self.conf.p2.z - offset as i64;
            self.nav.goto_head(&chest_loc, Order::XYZ);

            // Place items in chest
            for s in 0..max_chest_space.min(TURT_SLOTS) as usize {
                self.turt.inv_select(s as u8);
                self.turt.inv_drop_down();
            }
            // Check if chest didn't have enough slots for all items
            if max_chest_space < TURT_SLOTS {
                // Repeat
                chest_loc.z -= 1;
                self.nav.goto_head(&chest_loc, Order::XYZ);

                for s in max_chest_space..TURT_SLOTS {
                    self.turt.inv_select(s as u8);
                    self.turt.inv_drop_down();
                }
            };
            fstore_save(&self.fstore_chunk_digger);
            self.inv.full_update();

            // Return to mining position
            self.nav.goto_head(&saved_pos, Order::XYZ);
        }
    }

    /// All of p1's values are lower than p2.
    pub fn set_corners(&mut self, p1: Pos, p2: Pos) {
        if p1.x < p2.x {
            self.conf.p1.x = p1.x;
            self.conf.p2.x = p2.x;
        } else {
            self.conf.p1.x = p2.x;
            self.conf.p2.x = p1.x;
        }

        if p1.y < p2.y {
            self.conf.p1.y = p1.y;
            self.conf.p2.y = p2.y;
        } else {
            self.conf.p1.y = p2.y;
            self.conf.p2.y = p1.y;
        }

        if p1.z < p2.z {
            self.conf.p1.z = p1.z;
            self.conf.p2.z = p2.z;
        } else {
            self.conf.p1.z = p2.z;
            self.conf.p2.z = p1.z;
        }
    }

    pub fn save_progress(&self) {
        fstore_save(&self.fstore_chunk_digger)
    }

    /// All of p1's values are lower or equal to those of p2.
    pub fn run(&mut self) {
        let p1 = self.conf.p1.clone();
        let p2 = self.conf.p2.clone();
        let mut p = PosH::default();

        let y_diff = (p1.y.abs_diff(p2.y) / 3) as usize;
        let x_diff = p1.x.abs_diff(p2.x) as usize + 1;
        let z_diff = p1.z.abs_diff(p2.z) as usize + 1;

        let mut curr_slot = 0;

        for y in self.fstore_chunk_digger.layer..y_diff {
            p.y = p1.y + (y as i64 * 3) + 1;
            for x in 0..x_diff {
                if y % 2 == 0 {
                    p.x = p1.x + x as i64;
                } else {
                    p.x = p2.x - x as i64;
                }

                match (x_diff % 2 == 0, y % 2 == 0, x % 2 == 0) {
                    (false, false, false) => p.h = Head::N,
                    (false, false, true) => p.h = Head::S,
                    (false, true, false) => p.h = Head::S,
                    (false, true, true) => p.h = Head::N,
                    (true, false, false) => p.h = Head::S,
                    (true, false, true) => p.h = Head::N,
                    (true, true, false) => p.h = Head::S,
                    (true, true, true) => p.h = Head::N,
                }

                for z in 0..z_diff {
                    match p.h {
                        Head::N => p.z = p2.z - z as i64,
                        Head::S => p.z = p1.z + z as i64,
                        _ => panic!(),
                    }

                    self.nav.goto_head(&p, Order::XYZ);

                    self.turt.dig_up().unwrap();
                    self.turt.dig_down().unwrap();

                    match &self.conf.place_floor {
                        TurtBlock::None => (),
                        TurtBlock::Any => loop {
                            // Select non-empty slot and place
                            let slot = self.inv.reduce_count_andor_find_next(curr_slot);
                            if let Some(s) = slot {
                                let s = s as usize;
                                if curr_slot != s {
                                    curr_slot = s;
                                    self.turt.inv_select(curr_slot as u8);
                                }
                                self.turt.place_down();
                                break;
                            } else {
                                self.turt.print("Out of blocks! Please add more.");
                                std::thread::sleep(std::time::Duration::from_millis(1000));
                            }
                        }
                        TurtBlock::Some(_block) => {
                            unimplemented!("ChunkDigger::run: TurtBlock::Some(_block)")
                        }
                    }

                    if z % self.conf.check_inv_every_n_blocks == 0 {
                        self.inv_check();
                    }
                }
                self.inv_check();
            }
            self.fstore_chunk_digger.layer += 1;
            self.save_progress();
        }
        let mut chest_loc: PosH = p1.clone().into();
        chest_loc.z = self.conf.p2.z;
        self.nav.goto_head(&chest_loc, Order::XYZ);
    }
}
