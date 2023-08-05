//use crate::{nav, turtle::Turt};

//const MAX_BLOCKS: u8 = 64;

//pub async fn floor_placer(forward: i64, left: i64) {
    //let mut n = nav::Nav::new(1);
    //n.lpos();

    //let mut p = nav::Pos {
        //x: 0,
        //y: 0,
        //z: 0,
        //h: nav::Head::N,
    //};

    //let mut slot = 0;
    //let mut blocks = MAX_BLOCKS;
    
    //Turt::select(slot).await;
    //for x in 0..left {
        //p.x = -x;
        //if p.x % 2 == 0 {
            //p.h = nav::Head::N;
        //} else {
            //p.h = nav::Head::S;
        //}

        //for z in 0..forward as i64 {
            //if x % 2 == 0 {
                //p.z = -z;
            //} else {
                //p.z = -(forward - z - 1)
            //}

            //n.goto(&p, nav::Order::XYZ).await;

            //Turt::p_down().await;
            //blocks -= 1;
            //if blocks == 0 {
                //slot += 1;
                //slot %= 16;
                //blocks = MAX_BLOCKS;
                //Turt::select(slot).await;
            //}
        //}
    //}
    //p.x = 0;
    //p.z = 0;
    //p.h = nav::Head::N;
    //n.goto(&p, nav::Order::XYZ).await;
//}
