
use actix_web::{*,dev};
use actix::*;
use futures::*;
use tera;
use pixel_store;
use png_store;
use png;
use png_transform;

use futures;
use std::net::{IpAddr};
use structopt::StructOpt;
use std::path::PathBuf;
use vec_matrix;
use std::rc::*;

pub struct State {
    tera: tera::Tera,  // <- store tera template in application state
    pixels: Addr<Syn, pixel_store::PixelStore>,
    pngs: Addr<Syn, png_store::PNGStore>,
    index : Rc<String>,
    edit : Rc<String>,
    live : Rc<String>
}
impl State {
    pub fn new((width,height) : (usize,usize) , (nrows,ncols): (usize,usize)
        ,pixels: Addr<Syn, pixel_store::PixelStore>
        ,pngs: Addr<Syn, png_store::PNGStore>) -> State {
        let tera = compile_templates!("templates/**/*");
        let mut ctx = tera::Context::new();
        ctx.add("width", &width);
        ctx.add("height", &height);
        ctx.add("nrows", &nrows);
        ctx.add("ncols", &ncols);
        ctx.add("cell_width", &( width/ncols));
        ctx.add("cell_height", &( height/nrows));
        let index =  tera.render("index.html",&ctx).unwrap();
        let edit =  tera.render("edit.html",&ctx).unwrap();
        let live =  tera.render("live.html",&ctx).unwrap();
        
        State {
            tera,
            pngs,
            pixels,
            index : Rc::new(index),
            edit : Rc::new(edit),
            live : Rc::new(live)
        }
    }
}




pub fn index(req: HttpRequest<State>) -> Result<HttpResponse> {
   Ok(httpcodes::HTTPOk.build()
       .content_type("text/html")
        .body(req.state().index.clone())?)


}

pub fn edit_index(req: HttpRequest<State>) -> Result<HttpResponse>{
    Ok(httpcodes::HTTPOk.build()
       .content_type("text/html")
        .body(req.state().edit.clone())?)
}

pub fn live(req: HttpRequest<State>) -> Result<HttpResponse> {
    Ok(httpcodes::HTTPOk.build()
       .content_type("text/html")
        .body(req.state().live.clone())?)
}


pub fn save(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
     let (row,col) = match parse_cell_idx(&req) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let addr = req.state().pixels.clone();
        req.concat2()
            .from_err()
            .and_then(move |body| {
 
                let decoder = png::Decoder::new(body.as_ref());
                let (info, mut reader) = decoder.read_info().unwrap();
                let mut buf = vec![0; info.buffer_size()];
                
                let _ = reader.next_frame(&mut buf).unwrap();

                let bitmap = png_transform::RGBAPixels::from_vec(buf);
                addr.send(pixel_store::AddPartialBoard{
                    row,col,bitmap
                }).map_err(|v| panic!(v))
            })
            .and_then(|res| 
                match res {
                    Ok(_) => Ok(HttpResponse::build(StatusCode::OK)
                            .content_type("text/plain")
                            .body("ok").unwrap()),
                    Err(e) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                            .content_type("text/plain")
                            .body("Sad sad day").unwrap()),

                })
            .responder()
}

pub fn get_board(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
        req.state().pngs.send(png_store::GetBoard).map_err(|v| panic!(v))
            .and_then(|res| 
                match res {
                    Ok(data) => Ok(HttpResponse::build(StatusCode::OK)
                            .content_type("image/png")
                            .body( data ).unwrap()),
                    Err(e) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                            .content_type("text/plain")
                            .body("Sad sad day").unwrap()),

                })
            .responder()
}


fn parse_cell_idx( req: &HttpRequest<State>) -> Result<(usize,usize),Box<Future<Item=HttpResponse, Error=Error>> > {
    let row : usize = match req.match_info().query("row") {
            Ok(v) => v ,
            Err(e) => return Err( Box::new(futures::future::err(e.into())) )
        };
    let col : usize = match req.match_info().query("col") {
            Ok(v) => v ,
            Err(e) => return Err( Box::new(futures::future::err(e.into())) )
        };
    Ok((row,col))

}


pub fn get_partial_board(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
        let addr = req.state().pngs.clone();
        let (row,col) = match parse_cell_idx(&req) {
            Ok(v) => v,
            Err(e) => return e,
        };
       
        req.concat2()
            .from_err()
            .and_then(move |body| {
       
                addr.send(png_store::GetPartialBoard(row,col)).map_err(|v| panic!(v))
            })
            .and_then(|res| 
                match res {
                    Ok(data) => Ok(HttpResponse::build(StatusCode::OK)
                            .content_type("image/png")
                            .body(data.as_ref().to_vec()).unwrap()),
                    Err(e) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                            .content_type("text/plain")
                            .body("Sad sad day").unwrap()),

                })
            .responder()
}

pub fn clear(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
          let (row,col) = match parse_cell_idx(&req) {
            Ok(v) => v,
            Err(e) => return e,
        };
       
        let addr = req.state().pixels.clone();
        req.concat2()
            .from_err()
            .and_then(move |body| {

                addr.send(pixel_store::ClearPartialBoard(row,col)).map_err(|v| panic!(v))
            })
            .and_then(|res| 
                match res {
                    Ok(_) => Ok(HttpResponse::build(StatusCode::OK)
                            .content_type("text/plain")
                            .body("ok").unwrap()),
                    Err(e) => Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                            .content_type("text/plain")
                            .body("Sad sad day").unwrap()),

                })
            .responder()
}
