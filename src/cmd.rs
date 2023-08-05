use crate::{turtle, nav};

use rocket::serde::json::Value;
use serde::Deserialize;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

const TIMEOUT_ATTEMPTS: usize = 100;
const TIMEOUT_TIME: u64 = 100;

const MAX_TURTLES: usize = 48;

pub struct Control {
    pub id: usize,
    pub commands: Commands,
    pub turt: turtle::Turt,
}

impl Control {
    pub fn new(turtleid: usize) -> Self {
        Self {
            id: turtleid,
            commands: Commands::new(),
            turt: turtle::Turt::new(turtleid),
        }
    }
}

pub static COMMANDS: once_cell::sync::Lazy<Vec<Control>> = once_cell::sync::Lazy::new(|| {
    (0..=MAX_TURTLES)
        .into_iter()
        .map(|id| Control::new(id))
        .collect()
});

#[derive(Clone, Debug)]
pub enum Resp {
    BadReq(String),
    BadCode(String),
    Ok(Value),
}

#[derive(Deserialize, Debug)]
pub struct LuaResp {
    code: i32,
    out: Value,
}

impl Into<Resp> for LuaResp {
    fn into(self) -> Resp {
        match self.code {
            0 => Resp::Ok(self.out),
            -1 => Resp::BadReq(self.out.to_string()),
            -2 => Resp::BadCode(self.out.to_string()),
            _ => panic!("Invalid code!"),
        }
    }
}

pub struct Commands {
    turtle_registered: AtomicBool,

    cmd: Mutex<String>,
    http_ready: AtomicBool,
    http_done: AtomicBool,
    resp: Mutex<Resp>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            turtle_registered: AtomicBool::new(false),
            cmd: Mutex::new("".to_string()),
            http_ready: AtomicBool::new(false),
            http_done: AtomicBool::new(false),
            resp: Mutex::new(Resp::Ok(Value::Bool(true))),
        }
    }

    pub fn register(&self) {
        self.turtle_registered.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn turtle_registered(&self) -> bool {
        self.turtle_registered.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn run(&self, cmd: &str) -> Resp {
        println!("==== {cmd}");
        let mut l = self.cmd.lock().unwrap();
        *l = cmd.to_string();
        drop(l);

        self.http_ready
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.http_done
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let mut timed = false;
        for timeout in 0..TIMEOUT_ATTEMPTS {
            //dbg!(self.http_ready.load(std::sync::atomic::Ordering::SeqCst));
            if self.http_done.load(std::sync::atomic::Ordering::SeqCst) {
                timed = true;
                break;
            }
            if timeout == 0 {
                print!("Timeout ");
            }
            print!("=");
            std::io::stdout().flush().unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(TIMEOUT_TIME)).await;
        }
        if timed {
            println!()
        }

        self.http_ready
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let inner: Resp = self.resp.lock().unwrap().clone();
        return inner;
    }

    pub async fn disconnect(&self) {
        self.run("EXIT").await;
    }

    pub fn next(&self) -> String {
        if self.http_ready.load(std::sync::atomic::Ordering::SeqCst) {
            self.http_ready
                .store(false, std::sync::atomic::Ordering::SeqCst);
            return self.cmd.lock().unwrap().clone();
        } else {
            return "WAIT".to_string();
        }
    }

    pub fn completed(&self, resp: Resp) {
        self.http_done
            .store(true, std::sync::atomic::Ordering::SeqCst);
        //self.http_ready.store(false, std::sync::atomic::Ordering::SeqCst);

        let mut l = self.resp.lock().unwrap();
        *l = resp;
        drop(l);
    }
}
