use std::path::PathBuf;
use modelutils_rs::coords::Order;
use modelutils_rs::model2arr::{Block, CoordXZ, uint};
use crate::{DefaultData, PROGRESS_DIR, TurtleIdentifier};
use crate::scripts::model_builder::generation::{join_paths_greedily, mst_to_paths, nodes_to_mst};
use crate::turtle_core::control::TurtControl;
use crate::turtle_core::data::TurtResponse;
use crate::turtle_core::file_system_storage::{FStore, fstore_load_or_init, fstore_save};
use crate::turtle_core::inventory::{TURT_SLOTS, TurtInventory};
use crate::turtle_core::navigation::{Pos, PosH, TurtNavigation};

#[derive(Debug)]
struct FStoreModelBuilder {
    fp: PathBuf,
    start_layer: usize,
}

impl FStore for FStoreModelBuilder {
    fn default(p: &PathBuf) -> Self {
        Self {
            fp: p.clone(),
            start_layer: 0,
        }
    }

    fn path(&self) -> &PathBuf {
        &self.fp
    }

    fn save(&self) -> String {
        format!("{}\n", self.start_layer)
    }

    fn load(p: &PathBuf, d: &str) -> Self {
        let lines: Vec<String> = d.lines().map(String::from).collect();
        Self {
            fp: p.clone(),
            start_layer: lines[0].parse::<usize>().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelBuilderConfig {
    pub start_pos: Pos,
    pub max_chests: usize,
    pub allowed_blocks: Vec<String>,
}

#[derive(Debug)]
pub struct ModelBuilder<'a> {
    identifier: TurtleIdentifier,
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
            identifier: data.0,
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

    fn clear_inv(&mut self) {
        for s in 0..TURT_SLOTS {
            if let Some(block) = self.turt.inv_item_detail(s as u8) {
                if !self.conf.allowed_blocks.contains(&block.name().to_string()) {
                    self.turt.inv_select(s as u8);
                    self.turt.inv_drop_forw();
                }
            }
        }
        self.inv.full_update();
    }

    pub fn inv_update(&mut self, curr_slot: &mut u8) {
        if let Some(next_slot) = self.inv.reduce_count_andor_find_next(*curr_slot as usize) {
            *curr_slot = next_slot as u8;
            self.turt.inv_select(*curr_slot);
            return;
        }
        println!("Inventory empty! Refilling... [{}]", self.identifier);
        self.clear_inv();

        // Save position of turtle (to return to)
        let saved_pos = self.nav.pos().clone();

        // Calculate first chest location and go there
        let mut chest_loc: PosH = self.conf.start_pos.clone().into();
        chest_loc.x += self.index as i64;

        // Go to lowest y-level first and then to chest
        let mut next_pos = saved_pos.clone();
        next_pos.y = self.conf.start_pos.y;
        self.nav.goto_head(&next_pos, Order::XYZ);
        next_pos.x = chest_loc.x;
        self.nav.goto_head(&next_pos, Order::XYZ);
        self.nav.goto_head(&chest_loc, Order::XYZ);

        // Refill inventory
        let mut first = true;
        while !self.inv.is_full() {
            if !first {
                println!("Waiting for chest to refill... [{}]", self.identifier);
                std::thread::sleep(std::time::Duration::from_millis(10000));
            }
            first = false;
            let mut offset = 0;
            for _s in 0..TURT_SLOTS {
                let r = self.turt.suck_down();
                let success = match r {
                    TurtResponse::Ok(r) => {
                        r.as_array().unwrap()[0].as_bool().unwrap()
                    }
                    _ => false
                };
                if !success {
                    offset += 1;
                    offset %= self.conf.max_chests;
                    chest_loc.z = self.conf.start_pos.z - offset as i64;
                    self.nav.goto_head(&chest_loc, Order::XYZ);
                    continue;
                }
                self.clear_inv();
            }
        }

        // Go underneath building point
        next_pos.x = saved_pos.x;
        self.nav.goto_head(&next_pos, Order::XYZ);

        // Return to mining position
        self.nav.goto_head(&saved_pos, Order::XYZ);
    }


    fn curr_xz(&self) -> CoordXZ {
        let p = self.nav.pos();
        (p.x as uint, p.z as uint)
    }

    pub fn run(&mut self, nodes: &Vec<Vec<(CoordXZ, Block)>>, count: usize) {
        let num_chests = (count as f32 / 64.0 / self.conf.chest_slots as f32).ceil() as usize;
        let mut need_more_chests = match self.turt.inv_item_detail(0) {
            Some(chests) => {
                if chests.count() < num_chests as i32 {
                    true
                } else {
                    false
                }
            }
            None => true,
        };

        if !self.fstore_model_builder.start_layer == 0 {
            need_more_chests = true;
        } else {
            need_more_chests = false;
        }

        if need_more_chests {
            println!("Not enough chests! Need at least: {}", num_chests);
            std::thread::sleep(std::time::Duration::from_millis(10000));
            return;
        }

        for i in 0..num_chests {
            self.nav.goto_nohead(&Pos::new(
                self.conf.start_pos.x + self.index as i64,
                self.conf.start_pos.y,
                self.conf.start_pos.z - i as i64,
            ), Order::XYZ);
            self.inv.full_update();
            match self.inv.slots[0] {
                Some(ref mut chest) => {
                    if chest.name() != "minecraft:chest" {
                        println!("Slot 0 is not a chest!");
                        std::thread::sleep(std::time::Duration::from_millis(10000));
                        return;
                    }
                }
                None => {
                    println!("No chest in slot 0!");
                    std::thread::sleep(std::time::Duration::from_millis(10000));
                    return;
                }
            }
            self.turt.inv_select(0);
            self.turt.dig_down();
            self.turt.place_down();
        }

        fn world_coord(start: &Pos, coord: CoordXZ, y: usize) -> Pos {
            Pos {
                x: start.x + coord.0 as i64,
                y: start.y + y as i64,
                z: start.z + coord.1 as i64,
            }
        }

        let mut curr_slot: u8 = 0;

        for y in self.fstore_model_builder.start_layer..nodes.len() {
            let rev_y = nodes.len() - y - 1;
            let layer = &nodes[rev_y];
            if layer.is_empty() { continue; }

            self.fstore_model_builder.start_layer = y;
            self.save_progress();

            let mst = nodes_to_mst(&layer);
            let paths = mst_to_paths(mst);
            let path = join_paths_greedily(self.curr_xz(), paths, &layer);

            for node in path {
                self.inv_update(&mut curr_slot);

                let (coord, _block) = layer[node as usize];
                self.nav.goto_nohead(&world_coord(&self.conf.start_pos, coord, rev_y), Order::XYZ);

                self.turt.place_up();
            }
        }
    }
}