//use crate::{nav, turtle};

use rocket::serde::json::Value;
use serde::Deserialize;

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
