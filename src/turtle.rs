use crate::cmd;

use rocket::serde::json::Value;
use serde::Deserialize;

use std::sync::mpsc;

use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
pub struct Movement {
    success: bool,
    msg: Option<String>,
}

impl TryInto<Movement> for Value {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Movement, Self::Error> {
        let vals = self.as_array().ok_or(anyhow::anyhow!("Not an array"))?;

        if vals.len() < 1 || vals.len() > 2 {
            return Err(anyhow::anyhow!("Invalid response."));
        }

        let success = vals[0].as_bool().ok_or(anyhow::anyhow!("Not a bool"))?;
        if vals.len() == 1 {
            Ok(Movement { success, msg: None })
        } else {
            let msg = vals[1]
                .as_str()
                .ok_or(anyhow::anyhow!("No msg."))?
                .to_string();
            Ok(Movement {
                success,
                msg: Some(msg),
            })
        }
    }
}

impl Movement {
    pub fn success(&self) -> bool {
        self.success
    }

    pub fn msg(&self) -> &Option<String> {
        &self.msg
    }
}

impl TryFrom<cmd::Resp> for Movement {
    type Error = anyhow::Error;
    fn try_from(value: cmd::Resp) -> Result<Self, Self::Error> {
        match value {
            cmd::Resp::Ok(v) => v.try_into(),
            cmd::Resp::BadReq(e) => Err(anyhow::anyhow!(e)),
            cmd::Resp::BadCode(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TurtSlot {
    count: i32,
    name: String,
}

impl TryInto<TurtSlot> for Value {
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

impl TryFrom<cmd::Resp> for TurtSlot {
    type Error = anyhow::Error;
    fn try_from(value: cmd::Resp) -> Result<Self, Self::Error> {
        match value {
            cmd::Resp::Ok(v) => v.try_into(),
            cmd::Resp::BadReq(e) => Err(anyhow::anyhow!(e)),
            cmd::Resp::BadCode(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

impl TurtSlot {
    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug)]
pub struct Inspect {
    block: bool,
    name: Option<String>,
}

impl TryInto<Inspect> for Value {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Inspect, Self::Error> {
        let vals = self.as_array().ok_or(anyhow::anyhow!("Not an array"))?;

        if vals.len() != 2 {
            return Err(anyhow::anyhow!("Invalid response."));
        }

        let block = vals[0].as_bool().ok_or(anyhow::anyhow!("Not a bool"))?;
        if !block {
            Ok(Inspect { block, name: None })
        } else {
            let name = vals[1]["name"]
                .as_str()
                .ok_or(anyhow::anyhow!("No name."))?
                .to_string();
            Ok(Inspect {
                block,
                name: Some(name),
            })
        }
    }
}

impl Inspect {
    pub fn block(&self) -> bool {
        self.block
    }

    pub fn name(&self) -> &Option<String> {
        &self.name
    }
}

impl TryFrom<cmd::Resp> for Inspect {
    type Error = anyhow::Error;
    fn try_from(value: cmd::Resp) -> Result<Self, Self::Error> {
        match value {
            cmd::Resp::Ok(v) => v.try_into(),
            cmd::Resp::BadReq(e) => Err(anyhow::anyhow!(e)),
            cmd::Resp::BadCode(e) => Err(anyhow::anyhow!(e)),
        }
    }
}

/// m_ == Move
/// t_ == Turn
/// i_ == Inspect
#[derive(Debug)]
pub struct Turt<'a> {
    next_tx: mpsc::Sender<String>,
    cmdcomplete_rx: &'a mpsc::Receiver<cmd::Resp>,
}

impl<'a> Turt<'a> {
    pub fn new(
        next_tx: mpsc::Sender<String>,
        cmdcomplete_rx: &'a mpsc::Receiver<cmd::Resp>,
    ) -> Self {
        Self {
            next_tx,
            cmdcomplete_rx,
        }
    }

    fn make_req_t<T>(&self, cmd: &str) -> Result<T, <T as TryFrom<cmd::Resp>>::Error>
    where
        T: TryFrom<cmd::Resp>,
    {
        self.next_tx.send(cmd.to_string()).unwrap();
        let resp = self.cmdcomplete_rx.recv().unwrap();
        T::try_from(resp)
    }

    fn make_req(&self, cmd: &str) -> cmd::Resp {
       self.next_tx.send(cmd.to_string()).unwrap();
        self.cmdcomplete_rx.recv().unwrap()
    }

    pub fn m_forw(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.forward()")
    }

    pub fn m_back(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.back()")
    }

    pub fn m_up(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.up()")
    }

    pub fn m_down(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.down()")
    }

    pub fn t_left(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.turnLeft()")
    }

    pub fn t_right(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.turnRight()")
    }

    pub fn d_forw(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.dig()")
    }

    pub fn d_down(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.digDown()")
    }

    pub fn d_up(&self) -> anyhow::Result<Movement> {
        self.make_req_t("turtle.digUp()")
    }

    pub fn i_forw(&self) -> anyhow::Result<Inspect> {
        self.make_req_t("turtle.inspect()")
    }

    pub fn i_up(&self) -> anyhow::Result<Inspect> {
        self.make_req_t("turtle.inspectUp()")
    }

    pub fn i_down(&self) -> anyhow::Result<Inspect> {
        self.make_req_t("turtle.inspectDown()")
    }

    /// 0-indexed
    pub fn inv_select(&self, slot: u8) -> cmd::Resp {
        if slot > 15 {
            panic!("Invalid slot number!");
        }
        self.make_req(&format!("turtle.select({})", slot + 1))
    }

    pub fn inv_item_detail(&self, slot: u8) -> Option<TurtSlot> {
        if slot > 15 {
            panic!("Invalid slot number!");
        }
        match self.make_req_t::<TurtSlot>(&format!("turtle.getItemDetail({})", slot + 1)) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn inv_drop_forw(&self) -> cmd::Resp {
        self.make_req("turtle.drop()")
    }

    pub fn inv_drop_down(&self) -> cmd::Resp {
        self.make_req("turtle.dropDown()")
    }

    pub fn inv_drop_up(&self) -> cmd::Resp {
        self.make_req("turtle.dropUp()")
    }

    pub fn p_forw(&self) -> cmd::Resp {
        self.make_req("turtle.place()")
    }

    pub fn p_up(&self) -> cmd::Resp {
        self.make_req("turtle.placeUp()")
    }

    pub fn p_down(&self) -> cmd::Resp {
        self.make_req("turtle.placeDown()")
    }

    pub fn disconnect(&self) {
        self.next_tx.send("EXIT".to_string()).unwrap();
    }
}
