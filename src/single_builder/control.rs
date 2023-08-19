use crate::{turtle, nav, inventory};

use super::simple_searcher::SimpleSearcher;


const TURT_INV: usize = 16;
const CHEST_SIZE: usize = 27;

pub struct BuildController<'a> {
    model: Vec<Vec<Vec<bool>>>,

    turt: &'a turtle::Turt<'a>,
    nav: &'a mut nav::Nav<'a>,
    inv: inventory::Inventory<'a>,

    start_pos: nav::PosH,

    stack_count: usize,
    blocks_placed: usize,
}

impl<'a> BuildController<'a> {
    pub fn new(model: Vec<Vec<Vec<bool>>>, _turtleid: usize, turt: &'a turtle::Turt<'a>, nav: &'a mut nav::Nav<'a>) -> Self {
        Self {
            model,
            turt,
            nav,
            inv: inventory::Inventory::init(&turt),
            stack_count: 0,
            blocks_placed: 0,
            start_pos: nav::PosH::default(),
        }
    }

    fn count_blocks(&self) -> usize {
        let mut c = 0;
        for x in 0..self.model.len() {
            for y in 0..self.model[0].len() {
                for z in 0..self.model[0][0].len() {
                    if self.model[x][y][z] {
                        c += 1;
                    }
                }
            }
        }
        c
    }

    pub fn refill_inv(&mut self) {
        if self.blocks_placed % 64 == 0 {
            let slot = (self.blocks_placed / 64) % 16;
            if slot == 0 {
                println!("Refilling inventory");
                let curr_height = self.nav.pos().y;
                let mut chest_loc = self.start_pos.clone();
                chest_loc.z = self.start_pos.z;
                chest_loc.y = curr_height + 2;
                println!("{:?}", &chest_loc);
                self.nav.goto_head(&chest_loc, nav::Order::YXZ);


                self.stack_count += TURT_INV;
                let offset = (self.stack_count - TURT_INV) / CHEST_SIZE;
                let max_chest_space = CHEST_SIZE - ((self.stack_count - TURT_INV) % CHEST_SIZE);


                chest_loc.y = self.start_pos.y;
                chest_loc.z = self.start_pos.z - offset as i64;
                dbg!(&offset);
                dbg!(&chest_loc);
                self.nav.goto_head(&chest_loc, nav::Order::XYZ);

                for s in 0..max_chest_space.min(TURT_INV) {
                    self.turt.inv_select(s as u8);
                    self.turt.suck_down();
                }
                dbg!(max_chest_space);
                if max_chest_space < TURT_INV {
                    chest_loc.z -= 1;
                    self.nav.goto_head(&chest_loc, nav::Order::XYZ);

                    for s in max_chest_space..TURT_INV {
                        self.turt.inv_select(s as u8);
                        self.turt.suck_down();
                    }
                }
            }
            self.turt.inv_select(slot as u8);
        }
    }

    pub fn run(&mut self) {
        self.stack_count = 130; // TODO REMOVE
        const START_HEIGHT: usize = 42;


        println!("Running build controller");
        println!("#blocks {}", self.count_blocks());

        self.start_pos = self.nav.pos().clone();
        for y in START_HEIGHT..self.model[0].len() {
            let nodes = {
                let mut nodes = vec![];
                for x in 0..self.model.len() {
                    for z in 0..self.model[0][0].len() {
                        if self.model[x][y][z] {
                            nodes.push((x, z));
                        }
                    }
                }
                nodes
            };

            let mut searcher = SimpleSearcher::new(nodes);

            if searcher.nodes.len() == 0 {
                continue;
            }

            let mut curr = 0;
            while let Some(next) = searcher.next_node(curr) {
                curr = next;

                let node = searcher.nodes[curr].1;
                let pos = nav::Pos {
                    x: self.start_pos.x + node.0 as i64,
                    y: self.start_pos.y + y as i64 + 1,
                    z: self.start_pos.z + node.1 as i64,
                };

                self.refill_inv();

                self.nav.goto_nohead(&pos, nav::Order::XYZ);
                self.turt.p_down();
                self.blocks_placed += 1;
            }
        }
    }
}