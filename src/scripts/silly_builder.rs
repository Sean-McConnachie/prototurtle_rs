use crate::{inventory, nav, turtle};

use std::io::Write;
use std::path;

const TURT_INV: usize = 16;
const CHEST_SIZE: usize = 27;

#[derive(Debug)]
pub struct SillyBuilder<'a> {
    blocks: Vec<Vec<Vec<bool>>>,

    p1: nav::Pos,
    p2: nav::Pos,

    turt: &'a turtle::Turt<'a>,
    nav: &'a mut nav::Nav<'a>,
    inv: inventory::Inventory<'a>,

    step: usize,
    stack_count: usize,

    fp: path::PathBuf,
}

impl<'a> SillyBuilder<'a> {
    pub fn init(blocks: Vec<Vec<Vec<bool>>>, turtleid: usize, turt: &'a turtle::Turt<'a>, nav: &'a mut nav::Nav<'a>) -> Self {
        Self {
            blocks,

            p1: nav::Pos::default(),
            p2: nav::Pos::default(),

            turt,
            nav,

            inv: inventory::Inventory::init(&turt),

            step: 0,
            stack_count: 0,

            fp: path::PathBuf::from(format!("progress/{turtleid}.turtle")),
        }
    }

    fn inv_check(&mut self) {
        self.inv.full_update();
        if self.inv.is_full() {
            self.stack_count += TURT_INV;
            let spos = self.nav.pos().clone();

            let offset = (self.stack_count - TURT_INV) / CHEST_SIZE;
            let max_chest_space = CHEST_SIZE - ((self.stack_count - TURT_INV) % CHEST_SIZE);

            let mut chest_loc: nav::PosH = self.p1.clone().into();
            chest_loc.z = self.p2.z - offset as i64;
            dbg!(&offset);
            dbg!(&chest_loc);
            self.nav.goto_head(&chest_loc, nav::Order::XYZ);

            for s in 0..max_chest_space.min(TURT_INV) {
                self.turt.inv_select(s as u8);
                self.turt.inv_drop_down();
            }
            dbg!(max_chest_space);
            if max_chest_space < TURT_INV {
                chest_loc.z -= 1;
                self.nav.goto_head(&chest_loc, nav::Order::XYZ);

                for s in max_chest_space..TURT_INV {
                    self.turt.inv_select(s as u8);
                    self.turt.inv_drop_down();
                }
            }

            self.nav.goto_head(&spos, nav::Order::XYZ);
        }
    }

    /// All of p1's values are lower than p2.
    pub fn set_corners(&mut self, p1: nav::Pos, p2: nav::Pos) {
        if p1.x < p2.x {
            self.p1.x = p1.x;
            self.p2.x = p2.x;
        } else {
            self.p1.x = p2.x;
            self.p2.x = p1.x;
        }

        if p1.y < p2.y {
            self.p1.y = p1.y;
            self.p2.y = p2.y;
        } else {
            self.p1.y = p2.y;
            self.p2.y = p1.y;
        }

        if p1.z < p2.z {
            self.p1.z = p1.z;
            self.p2.z = p2.z;
        } else {
            self.p1.z = p2.z;
            self.p2.z = p1.z;
        }
    }

    pub fn get_or_init_progress(&mut self) {
        if !self.fp.exists() {
            self.save_progress()
        } else {
            let lines: Vec<String> = std::fs::read_to_string(&self.fp)
                .unwrap()
                .lines()
                .map(String::from)
                .collect();
            self.step = lines[0].parse::<usize>().unwrap();
            self.stack_count = lines[1].parse::<usize>().unwrap();
        }
    }

    pub fn save_progress(&self) {
        let mut f = std::fs::File::create(&self.fp).unwrap();
        f.write(format!("{}\n{}\n", self.step, self.stack_count).as_bytes())
            .unwrap();
    }

    // North = neg z
    // East = pos x
    /// All of p1's values are lower than p2.
    pub fn run(&mut self) {
        // TODO: STACK COUNT AND PLACE CHESTS
        let mut p = nav::PosH::default();

        let y_diff = self.p1.y.abs_diff(self.p2.y) as usize + 1;
        let x_diff = self.p1.x.abs_diff(self.p2.x) as usize + 1;
        let z_diff = self.p1.z.abs_diff(self.p2.z) as usize + 1;

        let mut blocks_placed: usize = 0;

        for y in self.step..y_diff {
            p.y = self.p1.y + (y as i64) + 1;
            for x in 0..x_diff {
                if y % 2 == 0 {
                    p.x = self.p1.x + x as i64;
                } else {
                    p.x = self.p2.x - x as i64;
                }

                match (x_diff % 2 == 0, y % 2 == 0, x % 2 == 0) {
                    (false, false, false) => p.h = nav::Head::N,
                    (false, false, true) => p.h = nav::Head::S,
                    (false, true, false) => p.h = nav::Head::S,
                    (false, true, true) => p.h = nav::Head::N,
                    (true, false, false) => p.h = nav::Head::S,
                    (true, false, true) => p.h = nav::Head::N,
                    (true, true, false) => p.h = nav::Head::S,
                    (true, true, true) => p.h = nav::Head::N,
                }

                for z in 0..z_diff {
                    match p.h {
                        nav::Head::N => p.z = self.p2.z - z as i64,
                        nav::Head::S => p.z = self.p1.z + z as i64,
                        _ => panic!(),
                    }

                    let mut block = self.blocks[x][y][z].clone();
                    if block {
                        self.nav.goto_head(&p, nav::Order::XYZ);
                        if blocks_placed % 64 == 0 {
                            self.turt.inv_select(((blocks_placed / 64) % 16) as u8);
                        }
                        self.turt.p_down();
                        blocks_placed += 1;
                    }

                    // self.turt.d_up().unwrap();
                    // self.turt.d_down().unwrap();
                    //
                    // if z % 63 == 0 {
                    //     self.inv_check();
                    // }
                }
                // self.inv_check();
            }
            self.step += 1;
            self.save_progress();
        }
        let mut chest_loc: nav::PosH = self.p1.clone().into();
        chest_loc.z = self.p2.z;
        self.nav.goto_head(&chest_loc, nav::Order::XYZ);
    }
}
