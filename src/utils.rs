pub struct BlockIter<const X: i32, const Y: i32, const Z: i32> {
    x: i32,
    y: i32,
    z: i32,
}

impl<const X: i32, const Y: i32, const Z: i32> BlockIter<X, Y, Z> {
    pub fn new() -> BlockIter<X, Y, Z> {
        BlockIter { x: 0, y: 0, z: 0 }
    }
}

impl<const X: i32, const Y: i32, const Z: i32> Default for BlockIter<X, Y, Z> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const X: i32, const Y: i32, const Z: i32> Iterator for BlockIter<X, Y, Z> {
    type Item = (i32, i32, i32);
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.y >= Y {
            return None;
        } else {
            (self.x, self.y, self.z)
        };
        self.x += 1;
        if self.x >= X {
            self.x -= X;
            self.z += 1;
        }
        if self.z >= Z {
            self.z -= Z;
            self.y += 1
        }
        Some(out)
    }
}
