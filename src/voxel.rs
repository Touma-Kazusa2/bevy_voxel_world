use bevy::{prelude::*, render::primitives::Aabb};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldVoxel {
    Unset,
    Air,
    Solid(u8),
}

impl WorldVoxel {
    pub fn is_unset(&self) -> bool {
        *self == WorldVoxel::Unset
    }

    pub fn is_air(&self) -> bool {
        *self == WorldVoxel::Air
    }

    pub fn is_solid(&self) -> bool {
        matches!(self, WorldVoxel::Solid(_))
    }
}

impl Voxel for WorldVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == WorldVoxel::Air || *self == WorldVoxel::Unset {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for WorldVoxel {
    type MergeValue = u8;

    fn merge_value(&self) -> Self::MergeValue {
        match self {
            WorldVoxel::Solid(v) => *v,
            _ => 0,
        }
    }
}

pub(crate) trait VoxelAabb {
    fn ray_intersection(&self, ray: Ray) -> Option<(Vec3, Vec3)>;
}

impl VoxelAabb for Aabb {
    fn ray_intersection(&self, ray: Ray) -> Option<(Vec3, Vec3)> {
        let min = self.min();
        let max = self.max();

        let axis_tmin_tmax = |min: f32, max: f32, origin: f32, direction: f32| {
            let mut tmin = (min - origin) / direction;
            let mut tmax = (max - origin) / direction;

            if tmin > tmax {
                std::mem::swap(&mut tmin, &mut tmax);
            }

            (tmin, tmax)
        };

        let (tmin_x, tmax_x) = axis_tmin_tmax(min.x, max.x, ray.origin.x, ray.direction.x);
        let (tmin_y, tmax_y) = axis_tmin_tmax(min.y, max.y, ray.origin.y, ray.direction.y);
        let (tmin_z, tmax_z) = axis_tmin_tmax(min.z, max.z, ray.origin.z, ray.direction.z);

        let tmin = tmin_x.max(tmin_y).max(tmin_z);
        let tmax = tmax_x.min(tmax_y).min(tmax_z);

        if tmin < 0.0 && tmax < 0.0 {
            return None;
        }

        let mut intersection = ray.origin + ray.direction * tmin;

        if tmin < 0.0 {
            intersection = ray.origin;
        }

        let mut normal = (intersection - Vec3::from(self.center)) * 2.0;
        normal = vec3_floor_with_tolerance(normal.abs(), 0.001) * normal.signum();

        Some((intersection, normal))
    }
}

#[inline]
fn floor_with_tolerance(value: f32, tolerance: f32) -> f32 {
    if (value.ceil() - value).abs() >= tolerance {
        value.floor()
    } else {
        (value + tolerance).floor()
    }
}

#[inline]
fn vec3_floor_with_tolerance(value: Vec3, tolerance: f32) -> Vec3 {
    Vec3::new(
        floor_with_tolerance(value.x, tolerance),
        floor_with_tolerance(value.y, tolerance),
        floor_with_tolerance(value.z, tolerance),
    )
}
