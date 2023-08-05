use crate::cmd;

use rocket::serde::json::Value;
use serde::Deserialize;

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
pub struct Turt {
    id: usize
}

impl Turt {
    pub fn new(turtleid: usize) -> Self {
        Self {
            id: turtleid
        }
    }

    pub async fn m_forw(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.forward()").await)
    }

    pub async fn m_back(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.back()").await)
    }

    pub async fn m_up(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.up()").await)
    }

    pub async fn m_down(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.down()").await)
    }

    pub async fn t_left(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.turnLeft()").await)
    }

    pub async fn t_right(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.turnRight()").await)
    }

    pub async fn d_forw(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.dig()").await)
    }

    pub async fn d_down(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.digDown()").await)
    }

    pub async fn d_up(&self) -> anyhow::Result<Movement> {
        Movement::try_from(cmd::COMMANDS[self.id].commands.run("turtle.digUp()").await)
    }

    pub async fn i_forw(&self) -> anyhow::Result<Inspect> {
        Inspect::try_from(cmd::COMMANDS[self.id].commands.run("turtle.inspect()").await)
    }

    pub async fn i_up(&self) -> anyhow::Result<Inspect> {
        Inspect::try_from(cmd::COMMANDS[self.id].commands.run("turtle.inspect()").await)
    }

    pub async fn i_down(&self) -> anyhow::Result<Inspect> {
        Inspect::try_from(cmd::COMMANDS[self.id].commands.run("turtle.inspect()").await)
    }

    /// 0-indexed
    pub async fn select(&self, slot: u8) -> cmd::Resp {
        if slot > 15 {
            panic!("Invalid slot number!");
        }
        cmd::COMMANDS[self.id].commands.run(&format!("turtle.select({})", slot + 1)).await
    }

    pub async fn p_forw(&self) -> cmd::Resp {
        cmd::COMMANDS[self.id].commands.run("turtle.place()").await
    }

    pub async fn p_up(&self) -> cmd::Resp {
        cmd::COMMANDS[self.id].commands.run("turtle.placeUp()").await
    }

    pub async fn p_down(&self) -> cmd::Resp {
        cmd::COMMANDS[self.id].commands.run("turtle.placeDown()").await
    }
}
