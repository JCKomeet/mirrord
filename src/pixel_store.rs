use std::time::Duration;
use actix_web::*;
use actix::*;
use futures::*;

use vec_matrix::*;

use png_transform::*;


pub struct PixelStore{
    pixelscells: CellMatrix<[u8;4]> , 
    transformer: Addr<Syn, PNGTransformer>,
    is_dirty : bool,
    is_bussy : bool, 
    cells_received : usize
}

impl PixelStore {
    pub fn new(wh: (usize,usize) , cells: (usize,usize) ,transformer: Addr<Syn, PNGTransformer>) -> PixelStore {
        PixelStore{
            pixelscells : CellMatrix::new( [0,0,0,0], cells,wh ) 
            , transformer
            , is_dirty : true
            , is_bussy : false
            , cells_received : 0
        }
    }

    fn update(&mut self,ctx: &mut Context<Self> ) {
        self.is_dirty = true;
        if !self.is_bussy {
            ctx.notify(PushPixelData);
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        // ctx.run_later(Duration::new(3, 0), |act, ctx| {
        //     ctx.notify(Scroll);
        //     act.hb(ctx);
        // });
    }
}

// Provide Actor implementation for our actor
impl Actor for PixelStore {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Context<Self>) {
            if false {
                self.hb(ctx)
            }
    }
}



#[derive(Debug)]
pub struct AddPartialBoard{
    pub row : usize , 
    pub col : usize , 
    pub bitmap :  RGBAPixels
}

impl Message for AddPartialBoard {
    type Result = Result<(), String>;
}

impl Handler<AddPartialBoard> for PixelStore {
    type Result = Result<(), String>;

    fn handle(&mut self, AddPartialBoard {row,col,bitmap} : AddPartialBoard , ctx: &mut Context<Self>) -> Self::Result {
        if row >= self.pixelscells.rows() || col >= self.pixelscells.cols() { return Err("Fuck off ".to_string())}
    
        self.pixelscells.get_cell_iterator((row,col)).zip(bitmap.0.into_iter()).for_each(|(xy,px)| {
            if px[3] > 100 {
                self.pixelscells.data[xy] = px;
            }
        });
         self.update(ctx);
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClearPartialBoard(pub usize,pub usize);

impl Message for ClearPartialBoard {
    type Result = Result<(), String>;
}

impl Handler<ClearPartialBoard> for PixelStore {
    type Result = Result<(), String>;

    fn handle(&mut self, ClearPartialBoard(row,col) : ClearPartialBoard , ctx: &mut Context<Self>) -> Self::Result {
        if row >= self.pixelscells.rows() || col >= self.pixelscells.cols() { return Err("Fuck off ".to_string())}
    
        let pixels : Vec<[u8;4]> = self.pixelscells.get_partial((row,col));
                
        let vec = encode(pixels,self.pixelscells.cell_width(),self.pixelscells.cell_height());

        use std::fs::File;
        use std::io::prelude::*;

        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let timestamp  = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

        if let Err(e) = File::create(format!("./history/{}.{}.png",timestamp.as_secs(),timestamp.subsec_nanos()) )
                                .and_then(|mut v| v.write_all(&vec)) {
            println!("Error {:?}",e);
        }
        

        self.pixelscells.get_cell_iterator((row,col)).for_each(|xy| {
            self.pixelscells.data[xy] = [0,0,0,0];
        });
        self.update(ctx);
        Ok(())
    }
}


#[derive(Message)]
pub struct Scroll;

impl Handler<Scroll> for PixelStore {
    type Result = ();

    fn handle(&mut self, _: Scroll , ctx: &mut Context<Self>) {
        let wh = (self.pixelscells.data.width , self.pixelscells.data.height);
        let cells = (self.pixelscells.rows(),self.pixelscells.cols());


        let mut newcells = CellMatrix::new( [0,0,0,0], wh,cells ) ;
        {
            let original = &self.pixelscells;
            for r in 1..original.rows() {
                for c in 0..original.cols() {
                    let into_iter = original.get_cell_iterator((r-1,c));
                    let from_iter = original.get_cell_iterator((r,c));
                    for (from,into) in from_iter.zip(into_iter) {
                        newcells.data[into] = original.data[from]
                    }
                }
            }
        }
        self.is_dirty = true;
        self.pixelscells = newcells;
        info!("Scrolled");
        ctx.notify(PushPixelData);
    }
}

pub struct PushPixelData;
impl Message for PushPixelData {
    type Result = Result<(),()>;
}


impl Handler<PushPixelData> for PixelStore {
    type Result = Box<Future<Item=(), Error=()>>;

    fn handle(&mut self, _: PushPixelData , ctx: &mut Context<Self>) -> Self::Result {
        
        if self.is_dirty == false {
            self.is_bussy = false;
            return Box::new( future::ok(()) )
        }
        self.is_bussy = true;
        self.is_dirty = false;
        
        let address = ctx.unsync_address();
        let fut = self.transformer.send(TransformPixels(self.pixelscells.clone())).then(move |v| {
            address.do_send(PushPixelData);
            Ok(())
        });
        Box::new(fut)
    }
}
