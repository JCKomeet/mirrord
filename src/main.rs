#![feature(conservative_impl_trait)]
#![feature(universal_impl_trait)]
extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate futures;
extern crate env_logger;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate tera;

#[macro_use]
extern crate log;

extern crate png;

mod vec_matrix;

mod png_store;
mod pixel_store;
mod png_transform;

mod requests;
use requests::*;

use actix_web::*;
use actix::*;
use futures::{Future, Stream};



use std::net::{IpAddr};
use structopt::StructOpt;
use std::path::PathBuf;
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "a", long = "address",default_value = "0.0.0.0")]
    address: IpAddr,

    #[structopt(short = "p", long = "port",default_value = "8080")]
    port: u16,

   #[structopt(short = "w", long = "width",default_value = "1050")]
    width: usize,
    #[structopt(short = "h", long = "height",default_value = "1680")]
    height: usize,

    #[structopt(short = "r", long = "rows",default_value = "5")]
    nrows: usize,

    #[structopt(short = "c", long = "cols",default_value = "2")]
    ncols: usize,
}


fn main() {
    let opt = Opt::from_args();


    let (port,address) = (opt.port,opt.address);
    let wh = (opt.width,opt.height);
    let cells = (opt.nrows,opt.ncols);

    ::std::env::set_var("RUST_LOG", "MagicBoardServer=info");
    let _ = env_logger::init();
    let sys = actix::System::new("MagicBoard");

    let pngs : Addr<Syn, _> = png_store::PNGStore::new().start();

    let transformer : Addr<Syn, _> = png_transform::PNGTransformer{ pngs: pngs.clone() }.start();

    let pixels : Addr<Syn, _> = pixel_store::PixelStore::new(wh,cells,transformer.clone()).start();

    let _ = HttpServer::new(move || {
        let state = State::new( wh,cells,pixels.clone(),pngs.clone());
        Application::with_state(state)
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.f( index ) )
            .resource("/live", |r| r.f( live ) )
            .resource("/{row}/{col}/edit",  |r| r.f( edit_index )  )
            .resource("/{row}/{col}/clear",  |r| r.f(clear))
            .resource("/{row}/{col}/board.png", |r| r.f(get_partial_board))
            .resource("/{row}/{col}/save", |r| r.f(save))
            .resource("/board.png", |r| r.f(get_board))
        })
        .bind((address,port)).unwrap()
        .threads(2)
        .shutdown_timeout(1)
        .start();

    println!("Started http server: {} {} ",address,port);

    let _ = sys.run();
    
}

