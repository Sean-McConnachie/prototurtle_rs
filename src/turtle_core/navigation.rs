// Module definitions
pub use heading::Head;
pub use position::{Pos, PosH};

// Crate imports
use super::control::{TurtControl, TurtFunc};
use super::data::{TurtResponse, TurtInspect};

// External imports
use modelutils_rs::coords::{Axis, Order};
use rand::Rng;
use std::path::PathBuf;
use std::sync::mpsc;
use crate::turtle_core::data::TurtMovement;
use crate::turtle_core::file_system_storage::{FStore, fstore_load_or_init, fstore_save};
use crate::TurtleIdentifier;

pub const NAV_DIR: &str = "positions";

mod heading {
    #[derive(Debug, Clone)]
    pub enum Head {
        N,
        E,
        S,
        W,
    }

    impl ToString for Head {
        fn to_string(&self) -> String {
            match self {
                Head::N => "n".to_string(),
                Head::E => "e".to_string(),
                Head::S => "s".to_string(),
                Head::W => "w".to_string(),
            }
        }
    }

    impl Head {
        pub fn from_str(head: &str) -> anyhow::Result<Self> {
            Ok(match head {
                "n" => Head::N,
                "e" => Head::E,
                "s" => Head::S,
                "w" => Head::W,
                _ => return Err(anyhow::anyhow!("Invalid rotation: {}", head)),
            })
        }

        /// Returns 0 if heading is same
        /// Positive value for rotating clockwise (turnRight)
        /// Negative value for rotating counter-clockwise (turnLeft)
        pub fn diff(&self, h: &Head) -> i8 {
            match (self, h) {
                (Head::N, Head::N) => 0,
                (Head::N, Head::E) => 1,
                (Head::N, Head::S) => 2,
                (Head::N, Head::W) => -1,

                (Head::E, Head::N) => -1,
                (Head::E, Head::E) => 0,
                (Head::E, Head::S) => 1,
                (Head::E, Head::W) => 2,

                (Head::S, Head::N) => 2,
                (Head::S, Head::E) => -1,
                (Head::S, Head::S) => 0,
                (Head::S, Head::W) => 1,

                (Head::W, Head::N) => 1,
                (Head::W, Head::E) => 2,
                (Head::W, Head::S) => -1,
                (Head::W, Head::W) => 0,
            }
        }
    }
}

mod position {
    use crate::turtle_core::data::TurtResponse;
    use super::heading::Head;

    #[derive(Debug, Clone)]
    pub struct Pos {
        pub x: i64,
        pub y: i64,
        pub z: i64,
    }

    impl Pos {
        pub const ORIG: Self = Self { x: 0, y: 0, z: 0 };

        pub fn new(x: i64, y: i64, z: i64) -> Self {
            Self { x, y, z }
        }
    }

    impl Into<Pos> for &PosH {
        fn into(self) -> Pos {
            Pos {
                x: self.x,
                y: self.y,
                z: self.z,
            }
        }
    }

    impl std::fmt::Display for Pos {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({}, {}, {})", self.x, self.y, self.z)
        }
    }

    #[derive(Debug, Clone)]
    pub struct PosH {
        pub x: i64,
        pub y: i64,
        pub z: i64,
        pub h: Head,
    }

    impl Into<PosH> for Pos {
        fn into(self) -> PosH {
            PosH {
                x: self.x,
                y: self.y,
                z: self.z,
                h: Head::N,
            }
        }
    }

    impl Default for PosH {
        fn default() -> Self {
            Self {
                x: 0,
                y: 0,
                z: 0,
                h: Head::N,
            }
        }
    }

    impl Into<PosH> for rocket::serde::json::Value {
        fn into(self) -> PosH {
            let p = self.as_array().unwrap();
            PosH {
                x: p[0].as_i64().unwrap(),
                y: p[1].as_i64().unwrap(),
                z: p[2].as_i64().unwrap(),
                h: Head::N,
            }
        }
    }
}

#[derive(Debug)]
struct FStoreNav {
    p: PosH,
    fp: PathBuf,
}

impl FStore for FStoreNav {
    fn default(p: &PathBuf) -> Self {
        Self {
            p: PosH::default(),
            fp: p.clone(),
        }
    }

    fn path(&self) -> &PathBuf {
        &self.fp
    }

    fn save(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n",
            self.p.x,
            self.p.y,
            self.p.z,
            self.p.h.to_string()
        )
    }

    fn load(p: &PathBuf, d: &str) -> Self {
        let lines: Vec<String> = d.lines().map(String::from).collect();
        Self {
            p: PosH {
                x: lines[0].parse::<i64>().unwrap(),
                y: lines[1].parse::<i64>().unwrap(),
                z: lines[2].parse::<i64>().unwrap(),
                h: Head::from_str(lines[3].as_str()).unwrap(),
            },
            fp: p.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TurtNavigation<'a> {
    turt: &'a TurtControl<'a>,
    avoid_other_turtles: bool,
    next_tx: mpsc::Sender<String>,
    cmdcomplete_rx: &'a mpsc::Receiver<TurtResponse>,
    fstore_nav: FStoreNav,
}

impl std::fmt::Display for PosH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {})[{}]",
            self.x,
            self.y,
            self.z,
            self.h.to_string()
        )
    }
}

impl<'a> std::fmt::Display for TurtNavigation<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fstore_nav.p)
    }
}

impl<'a> TurtNavigation<'a> {
    pub fn new(
        turtleid: TurtleIdentifier,
        turt: &'a TurtControl,
        avoid_other_turtles: bool,
        next_tx: mpsc::Sender<String>,
        cmdcomplete_rx: &'a mpsc::Receiver<TurtResponse>,
    ) -> Self {
        let fp = std::path::PathBuf::from(
            format!("{}/{}.nav", NAV_DIR, turtleid));
        let fstore_nav = fstore_load_or_init(&fp);

        Self {
            turt,
            avoid_other_turtles,
            next_tx,
            cmdcomplete_rx,
            fstore_nav,
        }
    }


    fn make_req(&self, cmd: &str) -> TurtResponse {
        self.next_tx.send(cmd.to_string()).unwrap();
        self.cmdcomplete_rx.recv().unwrap()
    }

    pub fn pos(&self) -> &PosH {
        &self.fstore_nav.p
    }

    pub fn pos_save(&self) {
        // I use this seemingly pointless helper function to be able to find usages using the lsp
        fstore_save(&self.fstore_nav);
    }

    pub fn gps_init(&mut self) {
        let p1: PosH = match self.make_req("gps.locate()") {
            TurtResponse::Ok(v) => v.into(),
            _ => panic!("Oh oh... no gps here."),
        };

        self.mv_forw();

        self.fstore_nav.p = match self.make_req("gps.locate()") {
            TurtResponse::Ok(v) => v.into(),
            _ => panic!("This is bad..."),
        };

        self.fstore_nav.p.h = if self.fstore_nav.p.z < p1.z {
            Head::N
        } else if self.fstore_nav.p.z > p1.z {
            Head::S
        } else if self.fstore_nav.p.x < p1.x {
            Head::W
        } else {
            Head::E
        }
    }

    fn ignore_err<T, E>(_: Result<T, E>) -> () {}

    pub fn turn_head(&mut self, h: Head) {
        let r = self.fstore_nav.p.h.diff(&h);
        for _ in 0..r.abs() {
            match r < 0 {
                true => self.turn_left(),
                false => self.turn_right(),
            };
        }
        self.fstore_nav.p.h = h;
        self.pos_save();
    }

    pub fn turn_left(&mut self) {
        self.fstore_nav.p.h = match self.fstore_nav.p.h {
            Head::N => Head::W,
            Head::E => Head::N,
            Head::S => Head::E,
            Head::W => Head::S,
        };
        Self::ignore_err(self.turt.turn_left());
        self.pos_save();
    }

    pub fn turn_right(&mut self) {
        self.fstore_nav.p.h = match self.fstore_nav.p.h {
            Head::N => Head::E,
            Head::E => Head::S,
            Head::S => Head::W,
            Head::W => Head::N,
        };
        Self::ignore_err(self.turt.turn_right());
        self.pos_save();
    }

    fn avoid_turtle(
        &mut self,
        inspect: &TurtInspect,
        dig_func: TurtFunc<'a, anyhow::Result<TurtMovement>>,
    ) {
        if let Some(b) = inspect.block() {
            if !self.avoid_other_turtles {
                Self::ignore_err(dig_func(self.turt));
            } else {
                if b == "computercraft:turtle_normal" {
                    let mut rng = rand::thread_rng();
                    if rng.gen_range(0..2) == 0 {
                        self.mv_up();
                        self.mv_forw();
                        self.mv_down();
                    }
                } else {
                    Self::ignore_err(dig_func(self.turt));
                }
            }
        }
    }

    pub fn mv_forw(&mut self) {
        loop {
            match self.turt.insp_forw() {
                Ok(i) => {
                    self.avoid_turtle(&i, TurtControl::dig_forw);
                }
                Err(_) => continue,
            }
            match self.turt.mv_forw() {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        match self.fstore_nav.p.h {
            Head::N => self.fstore_nav.p.z -= 1,
            Head::E => self.fstore_nav.p.x += 1,
            Head::S => self.fstore_nav.p.z += 1,
            Head::W => self.fstore_nav.p.x -= 1,
        }
        self.pos_save();
    }

    pub fn mv_back(&mut self) {
        match self.turt.mv_back() {
            Ok(m) => {
                if !m.success() {
                    return;
                }
            }
            Err(_) => return,
        }
        match self.fstore_nav.p.h {
            Head::N => self.fstore_nav.p.z += 1,
            Head::E => self.fstore_nav.p.x -= 1,
            Head::S => self.fstore_nav.p.z -= 1,
            Head::W => self.fstore_nav.p.x += 1,
        }
        self.pos_save();
    }

    pub fn mv_up(&mut self) {
        loop {
            match self.turt.insp_up() {
                Ok(i) => {
                    self.avoid_turtle(&i, TurtControl::dig_up);
                }
                Err(_) => continue,
            }
            match self.turt.mv_up() {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        self.fstore_nav.p.y += 1;
        self.pos_save();
    }

    pub fn mv_down(&mut self) {
        loop {
            match self.turt.insp_down() {
                Ok(i) => {
                    self.avoid_turtle(&i, TurtControl::dig_down);
                }
                Err(_) => continue,
            }
            match self.turt.mv_down() {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        self.fstore_nav.p.y -= 1;
        self.pos_save();
    }

    pub fn goto_head(&mut self, dst: &PosH, order: Order) {
        self.goto_nohead(&dst.into(), order);
        self.turn_head(dst.h.clone());
    }

    pub fn goto_nohead(&mut self, dst: &Pos, order: Order) {
        let order_arr = order.order_arr();
        for d in order_arr {
            match d {
                Axis::X => {
                    if self.fstore_nav.p.x < dst.x {
                        self.turn_head(Head::E);
                    } else if self.fstore_nav.p.x > dst.x {
                        self.turn_head(Head::W);
                    }
                    for _ in 0..(self.fstore_nav.p.x - dst.x).abs() as usize {
                        self.mv_forw();
                    }
                }
                Axis::Y => {
                    if self.fstore_nav.p.y < dst.y {
                        for _ in 0..(self.fstore_nav.p.y - dst.y).abs() as usize {
                            self.mv_up()
                        }
                    } else if self.fstore_nav.p.y > dst.y {
                        for _ in 0..(self.fstore_nav.p.y - dst.y).abs() as usize {
                            self.mv_down()
                        }
                    };
                }
                Axis::Z => {
                    if self.fstore_nav.p.z < dst.z {
                        self.turn_head(Head::S);
                    } else if self.fstore_nav.p.z > dst.z {
                        self.turn_head(Head::N);
                    }
                    for _ in 0..(self.fstore_nav.p.z - dst.z).abs() as usize {
                        self.mv_forw();
                    }
                }
            }
        }
        if self.avoid_other_turtles {
            if self.fstore_nav.p.x != dst.x || self.fstore_nav.p.y != dst.y || self.fstore_nav.p.z != dst.z {
                self.goto_nohead(&dst, order);
            }
        }
    }
}
