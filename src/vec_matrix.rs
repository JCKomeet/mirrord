
#[derive(Clone)]
pub struct VecMatrix<T>{
    pub width:usize,
    pub height:usize,
    pub data: Vec<T>
}
impl<T:Clone> VecMatrix<T> {
    pub fn new(val: T,width:usize,height:usize) -> VecMatrix<T> {
        VecMatrix {
            width, height , data: vec![val;width*height]
        }
    }
}
impl<T:Copy> VecMatrix<T> {
    pub fn get_px(&self,(row,col): (usize,usize)) -> T {
        self.data[row*self.width + col]
    }
}


use std::ops::{Index,IndexMut};
impl<T> Index<(usize,usize)> for VecMatrix<T> {
    type Output = T;
    fn index<'a>(&'a self, (row,col): (usize,usize)) -> &'a T {
        &self.data[row*self.width + col]
    }
}

impl<T>  IndexMut<(usize,usize)> for VecMatrix<T> {
    fn index_mut<'a>(&'a mut self,  (row,col): (usize,usize)) -> &'a mut T {
        &mut self.data[row*self.width + col]
    }
}

#[derive(Clone)]
pub struct CellMatrix<T>{
    rows:usize,
    cols:usize,
    pub data: VecMatrix<T>
}
impl<T:Clone+ Copy> CellMatrix<T> {
    pub fn new(val: T,(rows,cols) : (usize,usize) , (width,height) : (usize,usize) ) -> CellMatrix<T> {
        CellMatrix {
            rows , cols ,
            data: VecMatrix::new(val,width,height)
        }
    }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn cell_width(&self) -> usize { self.data.width / self.cols }
    pub fn cell_height(&self) -> usize { self.data.height / self.rows }
    pub fn get_cell_iterator(&self , (row,col) : (usize,usize) ) -> impl Iterator<Item=(usize,usize)> {
        let cell_height = self.cell_height();
        let cell_width = self.cell_width();
        let start_col = cell_width*col;
        let start_row = cell_height*row;

        ( start_row..(start_row + cell_height) ) .flat_map(move |v| ::std::iter::repeat(v).zip(  start_col..(start_col+cell_width)  ))
    }
    pub fn get_partial(&self, row_col: (usize,usize)) -> Vec<T> {
        self.get_cell_iterator(row_col).map(|xy|self.data.get_px(xy) ).collect()
    }

    
}

