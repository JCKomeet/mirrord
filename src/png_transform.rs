

use futures::Future;
use actix::*;

use std::sync::Arc;
use std::collections::HashMap;
use png;
use vec_matrix::*;

use png_store::*;




pub struct PNGTransformer{
    pub pngs: Addr<Syn, PNGStore>,
}

// Provide Actor implementation for our actor
impl Actor for PNGTransformer {
    type Context = Context<Self>;

}



//    pub png: Addr<Syn, PNGStore>

pub struct TransformPixels(pub CellMatrix<[u8;4]>);
impl Message for TransformPixels {
    type Result = Result<(),()>;
}

impl Handler<TransformPixels> for PNGTransformer {
    type Result = Box<Future<Item=(), Error=()>>;

    fn handle(&mut self, msg : TransformPixels , ctx: &mut Context<Self>) -> Self::Result {
        let png_store = self.pngs.clone();


        let cells : CellMatrix<[u8;4]> = msg.0;
        let cell_width = cells.cell_width();
        let cell_height = cells.cell_height();

        let mut partials = HashMap::new();
        for r in 0..cells.rows() {
            for c in 0..cells.cols() {
                let pixels : Vec<[u8;4]> = cells.get_partial((r,c));
                
                let vec = encode(pixels,cell_width,cell_height);

                partials.insert( (r,c) , Arc::new(vec) );
            }
        }
        let matrix = cells.data;
        let full = encode(matrix.data ,matrix.width ,matrix.height);
        debug!("Done transform");
        let msg = SetBoard{
            full: Arc::new(full),
            partials: partials
        };
            
        let fut = png_store.send(msg).map_err(|_| {
            info!("Could not deliver updates to png store");
        });
        Box::new(fut)
    }
}


#[derive(Debug)]
pub struct RGBAPixels(pub Vec<[u8;4]>);
impl RGBAPixels {
    pub fn from_vec(data: Vec<u8>) -> RGBAPixels {
        assert!(data.len() %4 == 0);
        assert!(data.capacity() % 4 == 0);
        RGBAPixels( data.chunks(4).map(|v| [v[0],v[1],v[2],v[3]]).collect() )
    }
}


pub fn encode(pixels : Vec<[u8;4]>,width: usize, height:usize) -> Vec<u8> {
    use png::HasParameters;
    let raw = cast_vec(pixels);
     let mut vec = Vec::with_capacity(4000);
    {
        let mut encoder = png::Encoder::new(&mut vec, width as u32, height as u32); // Width is 2 pixels and height is 1.
        encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&raw).unwrap(); // Save
    }

    vec.shrink_to_fit();
    vec
}

pub fn cast_vec(mut data: Vec<[u8;4]> ) -> Vec<u8> {
    let mut v = Vec::with_capacity(data.len()*2);
    for d in data.into_iter() {
        v.push(d[0]);
        v.push(d[1]);
        v.push(d[2]);
        v.push(d[3]);
    }
    v
}