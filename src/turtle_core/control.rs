use std::sync::mpsc;

use super::data::{TurtResponse, TurtMovement, TurtInspect, TurtSlot};
use anyhow::Result;

pub type TurtFunc<'a, R> = fn(&'a TurtControl<'a>) -> R;

#[derive(Debug)]
pub struct TurtControl<'a> {
    next_tx: mpsc::Sender<String>,
    cmdcomplete_rx: &'a mpsc::Receiver<TurtResponse>,
}

impl<'a> TurtControl<'a> {
    pub fn new(
        next_tx: mpsc::Sender<String>,
        cmdcomplete_rx: &'a mpsc::Receiver<TurtResponse>,
    ) -> Self {
        Self {
            next_tx,
            cmdcomplete_rx,
        }
    }

    pub fn make_req_t<T>(&self, cmd: &str) -> Result<T, <T as TryFrom<TurtResponse>>::Error>
        where
            T: TryFrom<TurtResponse>,
    {
        self.next_tx.send(cmd.to_string()).unwrap();
        let resp = self.cmdcomplete_rx.recv().unwrap();
        T::try_from(resp)
    }

    pub fn make_req(&self, cmd: &str) -> TurtResponse {
        self.next_tx.send(cmd.to_string()).unwrap();
        self.cmdcomplete_rx.recv().unwrap()
    }

    pub fn suck(&self) -> TurtResponse {
        self.make_req("turtle.suck()")
    }

    pub fn suck_up(&self) -> TurtResponse {
        self.make_req("turtle.suckUp()")
    }

    pub fn suck_down(&self) -> TurtResponse {
        self.make_req("turtle.suckDown()")
    }

    pub fn mv_forw(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.forward()")
    }

    pub fn mv_back(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.back()")
    }

    pub fn mv_up(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.up()")
    }

    pub fn mv_down(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.down()")
    }

    pub fn turn_left(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.turnLeft()")
    }

    pub fn turn_right(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.turnRight()")
    }

    pub fn dig_forw(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.dig()")
    }

    pub fn dig_down(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.digDown()")
    }

    pub fn dig_up(&self) -> Result<TurtMovement> {
        self.make_req_t("turtle.digUp()")
    }

    pub fn insp_forw(&self) -> Result<TurtInspect> {
        self.make_req_t("turtle.inspect()")
    }

    pub fn insp_up(&self) -> Result<TurtInspect> {
        self.make_req_t("turtle.inspectUp()")
    }

    pub fn insp_down(&self) -> Result<TurtInspect> {
        self.make_req_t("turtle.inspectDown()")
    }

    /// 0-indexed
    pub fn inv_select(&self, slot: u8) -> TurtResponse {
        if slot > 15 {
            panic!("Invalid slot number!");
        }
        self.make_req(&format!("turtle.select({})", slot + 1))
    }

    /// 0-indexed
    pub fn inv_item_detail(&self, slot: u8) -> Option<TurtSlot> {
        if slot > 15 {
            panic!("Invalid slot number!");
        }
        match self.make_req_t::<TurtSlot>(&format!("turtle.getItemDetail({})", slot + 1)) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn inv_drop_forw(&self) -> TurtResponse {
        self.make_req("turtle.drop()")
    }

    pub fn inv_drop_down(&self) -> TurtResponse {
        self.make_req("turtle.dropDown()")
    }

    pub fn inv_drop_up(&self) -> TurtResponse {
        self.make_req("turtle.dropUp()")
    }

    pub fn place_forw(&self) -> TurtResponse {
        self.make_req("turtle.place()")
    }

    pub fn place_up(&self) -> TurtResponse {
        self.make_req("turtle.placeUp()")
    }

    pub fn place_down(&self) -> TurtResponse {
        self.make_req("turtle.placeDown()")
    }

    pub fn print(&self, msg: &str) -> TurtResponse {
        self.make_req(format!("print(\"{msg}\")").as_str())
    }

    pub fn disconnect(&self) {
        self.next_tx.send("EXIT".to_string()).unwrap();
    }
}
