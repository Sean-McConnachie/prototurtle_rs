use std::path::PathBuf;
use modelutils_rs::coords::Order;
use modelutils_rs::model2arr::{Block, CoordXZ};
use crate::{DefaultData, PROGRESS_DIR};
use crate::turtle_core::control::TurtControl;
use crate::turtle_core::file_system_storage::{FStore, fstore_load_or_init, fstore_save};
use crate::turtle_core::inventory::{TURT_SLOTS, TurtInventory};
use crate::turtle_core::navigation::{Pos, PosH, TurtNavigation};

#[derive(Debug)]
struct FStoreModelBuilder {
    fp: PathBuf,
    start_layer: usize,
    stack_count: usize,
}

impl FStore for FStoreModelBuilder {
    fn default(p: &PathBuf) -> Self {
        Self {
            fp: p.clone(),
            start_layer: 0,
            stack_count: 0,
        }
    }

    fn path(&self) -> &PathBuf {
        &self.fp
    }

    fn save(&self) -> String {
        format!("{}\n{}\n", self.start_layer, self.stack_count)
    }

    fn load(p: &PathBuf, d: &str) -> Self {
        let lines: Vec<String> = d.lines().map(String::from).collect();
        Self {
            fp: p.clone(),
            start_layer: lines[0].parse::<usize>().unwrap(),
            stack_count: lines[1].parse::<usize>().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelBuilderConfig {
    pub start_pos: Pos,
    pub chest_slots: usize,
}

#[derive(Debug)]
pub struct ModelBuilder<'a> {
    _identifier: String,
    index: usize,
    turt: &'a TurtControl<'a>,
    nav: &'a mut TurtNavigation<'a>,
    inv: TurtInventory<'a>,

    conf: ModelBuilderConfig,
    fstore_model_builder: FStoreModelBuilder,
}

impl<'a> ModelBuilder<'a> {
    pub fn new(
        data: DefaultData<'a>, conf: ModelBuilderConfig,
    ) -> Self {
        let fp = PathBuf::from(
            format!("{}/{}.modelbuilder", PROGRESS_DIR, data.0));
        let fstore_model_builder = fstore_load_or_init::<FStoreModelBuilder>(&fp);
        Self {
            _identifier: data.0,
            index: data.1,
            turt: data.2,
            nav: data.3,
            inv: TurtInventory::init(&data.2),
            conf,
            fstore_model_builder,
        }
    }

    pub fn save_progress(&self) {
        fstore_save(&self.fstore_model_builder)
    }

    pub fn update_inv(&mut self, blocks_placed: usize) {
        // if blocks_placed % 64 == 0 {
        //     let slot = (blocks_placed / 64) % 16;
        //     if slot == 0 {
        //         let chest_slots = self.conf.chest_slots;
        //         let mut chest_loc = self.conf.start_pos.clone();
        //         chest_loc.x += self.index as i64;
        //
        //         self.fstore_model_builder.stack_count += TURT_SLOTS;
        //         let offset = ((self.stack_count as f32 / TURT_SLOTS as f32) / chest_slots as f32) as i64;
        //         let max_chest_space = chest_slots - ((self.stack_count - TURT_SLOTS) % chest_slots);
        //
        //         chest_loc.z -= offset;
        //         self.nav.goto_head(&chest_loc, Order::XYZ);
        //
        //         for i in 0..TURT_SLOTS {
        //             self.turt.inv_select(i as u8);
        //             self.turt.suck_down();
        //         }
        //
        //         if max_chest_space < TURT_SLOTS {
        //             chest_loc.z -= 1;
        //             self.nav.goto_head(&chest_loc, Order::XYZ);
        //
        //             for i in max_chest_space..TURT_SLOTS {
        //                 self.turt.inv_select(i as u8);
        //                 self.turt.suck_down();
        //             }
        //         }
        //         self.inv.full_update();
        //         while !self.inv.is_full() {
        //             std::thread::sleep(std::time::Duration::from_millis(10000));
        //             println!("Refill turtle: {}", self.turt_i)
        //         }
        //     }
        //     self.turt.inv_select(slot as u8);
        // }
    }

    pub fn run(&mut self, nodes: &Vec<Vec<(CoordXZ, Block)>>, count: usize) {
        // let num_chests = (count as f32 / SLOT_SIZE as f32 / self.conf.chest_slots as f32).ceil() as usize;
        // let mut need_more_chests = match self.turt.inv_item_detail(0) {
        //     Some(chests) => {
        //         if chests.count() < num_chests as i32 {
        //             true
        //         } else {
        //             false
        //         }
        //     }
        //     None => true,
        // };
        //
        // if !self.start_layer == 0 {
        //     need_more_chests = true;
        // }
        //
        // if need_more_chests {
        //     println!("Not enough chests! Need at least: {}", num_chests);
        //     std::thread::sleep(std::time::Duration::from_millis(10000));
        //     // return;
        // }
        //
        // for i in 0..num_chests {
        //     self.nav.goto_nohead(&Pos::new(
        //         self.start_pos.x + self.turt_i as i64,
        //         self.start_pos.y,
        //         self.start_pos.z - i as i64,
        //     ), Order::XYZ);
        //     self.turt.inv_select(0);
        //     self.turt.d_down();
        //     self.turt.p_down();
        // }
        //
        //
        // fn world_coord(start: &PosH, coord: CoordXZ, y: usize) -> Pos {
        //     Pos {
        //         x: start.x + coord.0 as i64,
        //         y: start.y + y as i64,
        //         z: start.z + coord.1 as i64,
        //     }
        // }
        //
        // let mut blocks_placed = 0;
        //
        // for y in (self.start_layer..nodes.len()).rev() {
        //     let layer = &nodes[y];
        //     if layer.is_empty() { continue; }
        //
        //     self.start_layer = y;
        //     self.save_progress();
        //
        //     let path = nodes_to_mst_to_path(&layer);
        //
        //     for node in path {
        //         self.update_inv(blocks_placed);
        //
        //         let (coord, _block) = layer[node as usize];
        //         self.nav.goto_nohead(&world_coord(&self.start_pos, coord, y), Order::XYZ);
        //         self.turt.p_up();
        //
        //         self.turt.p_up();
        //         blocks_placed += 1;
        //     }
        // }
    }
}