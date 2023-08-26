//! Data that is passed between the turtle (client) and server.
pub use inventory::TurtSlot;
pub use world::{TurtInspect, TurtMovement};

#[derive(Clone, Debug)]
pub enum TurtResponse {
    BadReq(String),
    BadCode(String),
    Ok(rocket::serde::json::Value),
}

#[derive(serde::Deserialize, Debug)]
pub struct TurtRawResponse {
    code: i32,
    out: rocket::serde::json::Value,
}

impl Into<TurtResponse> for TurtRawResponse {
    fn into(self) -> TurtResponse {
        match self.code {
            0 => TurtResponse::Ok(self.out),
            -1 => TurtResponse::BadReq(self.out.to_string()),
            -2 => TurtResponse::BadCode(self.out.to_string()),
            _ => panic!("Invalid code!"),
        }
    }
}

mod inventory {
    use super::TurtResponse;

    #[derive(serde::Deserialize, Debug, Clone)]
    pub struct TurtSlot {
        count: i32,
        name: String,
    }

    impl TurtSlot {
        pub fn count(&self) -> i32 {
            self.count
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn is_empty(&self) -> bool {
            self.count == 0
        }

        pub fn reduce_count(&mut self, n: i32) {
            self.count -= n;
        }
    }

    impl TryInto<TurtSlot> for rocket::serde::json::Value {
        type Error = anyhow::Error;
        fn try_into(self) -> Result<TurtSlot, Self::Error> {
            let s = self.as_array().unwrap();
            if s.len() == 0 {
                return Err(anyhow::anyhow!("No item"));
            }

            let o = s[0].as_object().unwrap();
            Ok(TurtSlot {
                count: o.get("count").unwrap().as_i64().unwrap() as i32,
                name: o.get("name").unwrap().as_str().unwrap().to_string(),
            })
        }
    }

    impl TryFrom<TurtResponse> for TurtSlot {
        type Error = anyhow::Error;
        fn try_from(value: TurtResponse) -> Result<Self, Self::Error> {
            match value {
                TurtResponse::Ok(v) => v.try_into(),
                TurtResponse::BadReq(e) => Err(anyhow::anyhow!(e)),
                TurtResponse::BadCode(e) => Err(anyhow::anyhow!(e)),
            }
        }
    }
}

mod world {
    use super::TurtResponse;

    #[derive(serde::Deserialize, Debug)]
    pub struct TurtMovement {
        success: bool,
        msg: Option<String>,
    }

    impl TurtMovement {
        pub fn success(&self) -> bool {
            self.success
        }

        pub fn msg(&self) -> &Option<String> {
            &self.msg
        }
    }

    impl TryInto<TurtMovement> for rocket::serde::json::Value {
        type Error = anyhow::Error;
        fn try_into(self) -> Result<TurtMovement, Self::Error> {
            let vals = self.as_array().ok_or(anyhow::anyhow!("Not an array"))?;

            if vals.len() < 1 || vals.len() > 2 {
                return Err(anyhow::anyhow!("Invalid response."));
            }

            let success = vals[0].as_bool().ok_or(anyhow::anyhow!("Not a bool"))?;
            if vals.len() == 1 {
                Ok(TurtMovement { success, msg: None })
            } else {
                let msg = vals[1]
                    .as_str()
                    .ok_or(anyhow::anyhow!("No msg."))?
                    .to_string();
                Ok(TurtMovement {
                    success,
                    msg: Some(msg),
                })
            }
        }
    }

    impl TryFrom<TurtResponse> for TurtMovement {
        type Error = anyhow::Error;
        fn try_from(value: TurtResponse) -> Result<Self, Self::Error> {
            match value {
                TurtResponse::Ok(v) => v.try_into(),
                TurtResponse::BadReq(e) => Err(anyhow::anyhow!(e)),
                TurtResponse::BadCode(e) => Err(anyhow::anyhow!(e)),
            }
        }
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct TurtInspect {
        block: Option<String>,
    }

    impl TurtInspect {
        pub fn block(&self) -> &Option<String> {
            &self.block
        }
    }

    impl TryInto<TurtInspect> for rocket::serde::json::Value {
        type Error = anyhow::Error;
        fn try_into(self) -> Result<TurtInspect, Self::Error> {
            let vals = self.as_array().ok_or(anyhow::anyhow!("Not an array"))?;

            if vals.len() != 2 {
                return Err(anyhow::anyhow!("Invalid response."));
            }

            let block = vals[0].as_bool().ok_or(anyhow::anyhow!("Not a bool"))?;
            if !block {
                Ok(TurtInspect { block: None })
            } else {
                let name = vals[1]["name"]
                    .as_str()
                    .ok_or(anyhow::anyhow!("No name."))?
                    .to_string();
                Ok(TurtInspect {
                    block: Some(name),
                })
            }
        }
    }

    impl TryFrom<TurtResponse> for TurtInspect {
        type Error = anyhow::Error;
        fn try_from(value: TurtResponse) -> Result<Self, Self::Error> {
            match value {
                TurtResponse::Ok(v) => v.try_into(),
                TurtResponse::BadReq(e) => Err(anyhow::anyhow!(e)),
                TurtResponse::BadCode(e) => Err(anyhow::anyhow!(e)),
            }
        }
    }
}

