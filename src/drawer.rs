extern crate nalgebra as na;

use nalgebra::{Vector2};

use std::collections::HashMap;

trait Drawable
{
    fn draw(drawer: &Drawer);
}

struct Drawer 
{
    zoom: f64,
    
    offset: Vector2<f64>,
}

impl Drawer 
{
    
}
