use std::io::{Seek, SeekFrom, Write};

use crate::turtle::Turt;

pub enum Order {
    XYZ,
    YXZ,
    XZY,
    YZX,
    ZXY,
    ZYX,
}

impl Order {
    pub fn order_arr(&self) -> (char, char, char) {
        match self {
            Self::XYZ => ('x', 'y', 'z'),
            Self::YXZ => ('y', 'x', 'z'),
            Self::XZY => ('x', 'z', 'y'),
            Self::YZX => ('y', 'z', 'x'),
            Self::ZXY => ('z', 'x', 'y'),
            Self::ZYX => ('z', 'y', 'x'),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Head {
    N,
    E,
    S,
    W,
}

impl Into<Head> for &str {
    fn into(self) -> Head {
        match self {
            "n" => Head::N,
            "e" => Head::E,
            "s" => Head::S,
            "w" => Head::W,
            _ => panic!("Invalid rotation: {}", self),
        }
    }
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
    /// Returns 0 if heading is same
    /// Positive value for rotating clockwise (turnRight)
    /// Negative value for rotating counter-clockwise (turnLeft)
    fn diff(&self, h: &Head) -> i8 {
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

#[derive(Debug, Clone)]
pub struct Pos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub h: Head,
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            h: Head::N,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nav {
    p: Pos,
    fp: std::path::PathBuf,
}

impl Nav {
    pub fn new(turtleid: i32) -> Self {
        let fp = std::path::PathBuf::from(format!("turtle_positions/{turtleid}.turtle"));
        let s = Self {
            p: Pos::default(),
            fp,
        };
        if !s.fp.exists() {
            s.spos();
        }
        s
    }

    pub fn pos(&self) -> &Pos {
        &self.p
    }

    pub fn spos(&self) {
        let mut f = std::fs::File::create(&self.fp).unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        f.write(
            format!(
                "{}\n{}\n{}\n{}\n",
                self.p.x,
                self.p.y,
                self.p.z,
                self.p.h.to_string()
            )
            .as_bytes(),
        )
        .unwrap();
    }

    pub fn lpos(&mut self) {
        let lines: Vec<String> = std::fs::read_to_string(&self.fp)
            .unwrap()
            .lines()
            .map(String::from)
            .collect();

        self.p.x = lines[0].parse::<i64>().unwrap();
        self.p.y = lines[1].parse::<i64>().unwrap();
        self.p.z = lines[2].parse::<i64>().unwrap();
        self.p.h = lines[3].as_str().into();
    }

    fn ignore_err<T, E>(_: Result<T, E>) -> () {}

    pub async fn t_head(&mut self, h: Head) {
        let r = self.p.h.diff(&h);
        for _ in 0..r.abs() {
            match r < 0 {
                true => self.t_left().await,
                false => self.t_right().await,
            };
        }
        self.p.h = h;
        self.spos();
    }

    pub async fn t_left(&mut self) {
        self.p.h = match self.p.h {
            Head::N => Head::W,
            Head::E => Head::N,
            Head::S => Head::E,
            Head::W => Head::S,
        };
        Self::ignore_err(Turt::t_left().await);
        self.spos();
    }

    pub async fn t_right(&mut self) {
        self.p.h = match self.p.h {
            Head::N => Head::E,
            Head::E => Head::S,
            Head::S => Head::W,
            Head::W => Head::N,
        };
        Self::ignore_err(Turt::t_right().await);
        self.spos();
    }

    pub async fn m_forw(&mut self) {
        loop {
            match Turt::i_forw().await {
                Ok(i) => {
                    if i.block() {
                        Self::ignore_err(Turt::d_forw().await)
                    }
                }
                Err(_) => continue,
            }
            match Turt::m_forw().await {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        match self.p.h {
            Head::N => self.p.z -= 1,
            Head::E => self.p.x += 1,
            Head::S => self.p.z += 1,
            Head::W => self.p.x -= 1,
        }
        self.spos();
    }

    pub async fn m_back(&mut self) {
        match Turt::m_back().await {
            Ok(m) => {
                if !m.success() {
                    return;
                }
            }
            Err(_) => return,
        }
        match self.p.h {
            Head::N => self.p.z += 1,
            Head::E => self.p.x -= 1,
            Head::S => self.p.z -= 1,
            Head::W => self.p.x += 1,
        }
        self.spos();
    }

    pub async fn m_up(&mut self) {
        loop {
            match Turt::i_up().await {
                Ok(i) => {
                    if i.block() {
                        Self::ignore_err(Turt::d_up().await)
                    }
                }
                Err(_) => continue,
            }
            match Turt::m_up().await {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        self.p.y += 1;
        self.spos();
    }

    pub async fn m_down(&mut self) {
        loop {
            match Turt::i_down().await {
                Ok(i) => {
                    if i.block() {
                        Self::ignore_err(Turt::d_down().await)
                    }
                }
                Err(_) => continue,
            }
            match Turt::m_down().await {
                Ok(m) => {
                    if m.success() {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
        self.p.y -= 1;
        self.spos();
    }

    pub async fn goto(&mut self, dst: &Pos, order: Order) {
        let order = order.order_arr();
        for d in order.0..=order.2 {
            match d {
                'x' => {
                    if self.p.x < dst.x {
                        self.t_head(Head::E).await;
                    } else if self.p.x > dst.x {
                        self.t_head(Head::W).await;
                    }
                    while self.p.x != dst.x {
                        self.m_forw().await;
                    }
                }
                'y' => {
                    if self.p.y < dst.y {
                        while self.p.y != dst.y {
                            self.m_up().await
                        }
                    } else if self.p.y > dst.y {
                        while self.p.y != dst.y {
                            self.m_down().await
                        }
                    };
                }
                'z' => {
                    if self.p.z < dst.z {
                        self.t_head(Head::S).await;
                    } else if self.p.z > dst.z {
                        self.t_head(Head::N).await;
                    }
                    while self.p.z != dst.z {
                        self.m_forw().await;
                    }
                }
                _ => panic!(),
            }
        }
        self.t_head(dst.h.clone()).await;
    }
}
