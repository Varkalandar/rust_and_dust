use std::cmp::Ordering;


pub struct Voxel
{
    pub x: f32,
    pub y: f32,
    pub z: f32,

    pub size: f32,

    pub color: [u8; 4],
}


impl Voxel
{
    pub fn new(x: f32, y: f32, z: f32, size: f32, color: [u8; 4]) -> Voxel
    {
        Voxel {
            x,
            y,
            z,
            size,
            color,
        }
    }
}


pub struct Voxelstack 
{
    pub voxels: Vec<Voxel>,
}


impl Voxelstack
{
    pub fn new() -> Voxelstack
    {
        Voxelstack {
            voxels: Vec::new(),
        }
    }


    pub fn add(&mut self, voxel: Voxel)
    {
        self.voxels.push(voxel);
    }


    pub fn merge(&mut self, voxels: Voxelstack)
    {
        for voxel in voxels.voxels {
            self.add(voxel);
        }
    }


    pub fn sort_depth_first(&mut self)
    {
        self.voxels.sort_unstable_by(|a, b| -> Ordering {
            if b.z > a.z {
                Ordering::Greater
            } else if b.z < a.z {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }


    pub fn rotate_y(&mut self, angle: f32)
    {
        let sin = angle.sin();
        let cos = angle.cos();

        for voxel in &mut self.voxels {

            let x = voxel.x * cos - voxel.z * sin;
            let z = voxel.x * sin + voxel.z * cos;

            voxel.x = x;
            voxel.z = z;
        } 
    }
}

