use crate::{entry, nav, turtle};

use std::io::Write;
use std::path;

#[derive(Debug)]
pub struct ChunkDigger<'a> {
    p1: nav::Pos,
    p2: nav::Pos,

    turt: &'a turtle::Turt<'a>,
    nav: &'a mut nav::Nav<'a>,

    step: usize,
    stack_count: usize,

    fp: path::PathBuf,
}

impl<'a> ChunkDigger<'a> {
    pub fn init(turtleid: usize, turt: &'a turtle::Turt<'a>, nav: &'a mut nav::Nav<'a>) -> Self {
        Self {
            p1: nav::Pos::default(),
            p2: nav::Pos::default(),

            turt,
            nav,

            step: 0,
            stack_count: 0,

            fp: path::PathBuf::from(format!("progress/{turtleid}.turtle")),
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

        let y_diff = (self.p1.y.abs_diff(self.p2.y) / 3) as usize;
        let x_diff = self.p1.x.abs_diff(self.p2.x) as usize + 1;
        let z_diff = self.p1.z.abs_diff(self.p2.z) as usize + 1;

        for y in self.step..y_diff {
            p.y = self.p1.y + (y as i64 * 3) + 1;
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

                    self.nav.goto(&p, nav::Order::XYZ);
                    self.turt.d_up().unwrap();
                    self.turt.d_down().unwrap();
                }
            }
            self.step += 1;
            self.save_progress();
        }
    }
}