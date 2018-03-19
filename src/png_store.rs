
use actix_web::*;
use actix::*;
use std::sync::Arc;
use std::collections::HashMap;




pub struct PNGStore{
    full: Arc< Vec<u8> >,
    partials: HashMap<(usize,usize), Arc< Vec<u8> > >
}
impl Actor for PNGStore {
    type Context = Context<Self>;
}
impl PNGStore {
    pub fn new() -> PNGStore {
        PNGStore {
            full: Arc::new(vec![] ),
            partials: Default::default()
        }
    }
}


#[derive(Message,Debug)]
pub struct SetBoard{
    pub full: Arc< Vec<u8> >,
    pub partials: HashMap<(usize,usize), Arc< Vec<u8> > >
}

impl Handler<SetBoard> for PNGStore {
    type Result = ();
    fn handle(&mut self, new_board : SetBoard , _: &mut Context<Self>) -> Self::Result {
        info!("Board updated");
        self.full = new_board.full;
        self.partials = new_board.partials;
    }
}



pub struct GetPartialBoard(pub usize,pub usize);
impl Message for GetPartialBoard {
    type Result = Result<Arc<Vec<u8>>, String>;
}


impl Handler<GetPartialBoard> for PNGStore {
    type Result = Result<Arc<Vec<u8>>, String>;

    fn handle(&mut self, GetPartialBoard(row,col) : GetPartialBoard , _: &mut Context<Self>) -> Self::Result {
        self.partials.get(&( row,col )).map(|v| v.clone()).ok_or_else (|| "Fuck off ".to_string() )
    }
}

pub struct GetBoard;
impl Message for GetBoard {
    type Result = Result<Arc<Vec<u8>>, ()>;
}

impl Handler<GetBoard> for PNGStore {
    type Result = Result<Arc<Vec<u8>>,()>;

    fn handle(&mut self, _: GetBoard , _: &mut Context<Self>) -> Self::Result {
        Ok(self.full.clone())
    }
}
