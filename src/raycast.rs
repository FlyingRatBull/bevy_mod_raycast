use std::f32::EPSILON;

use crate::primitives::*;
use bevy::prelude::*;

#[allow(dead_code)]
#[non_exhaustive]
pub enum RaycastAlgorithm {
    Geometric,
    MollerTrumbore(Backfaces),
}

impl Default for RaycastAlgorithm {
    fn default() -> Self {
        RaycastAlgorithm::MollerTrumbore(Backfaces::Cull)
    }
}

#[allow(dead_code)]
pub enum Backfaces {
    Cull,
    Include,
}

/// Takes a ray and triangle and computes the intersection and normal
pub fn ray_triangle_intersection(
    ray: &Ray3d,
    triangle: &Triangle,
    algorithm: RaycastAlgorithm,
) -> Option<Ray3d> {
    match algorithm {
        RaycastAlgorithm::Geometric => raycast_geometric(ray, triangle),
        RaycastAlgorithm::MollerTrumbore(backface_culling) => {
            raycast_moller_trumbore(ray, triangle, backface_culling)
        }
    }
}

/// Implementation of the Möller-Trumbore ray-triangle intersection test
pub fn raycast_moller_trumbore(
    ray: &Ray3d,
    triangle: &Triangle,
    backface_culling: Backfaces,
) -> Option<Ray3d> {
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection
    let vector_v0_to_v1: Vec3 = triangle.v1 - triangle.v0;
    let vector_v0_to_v2: Vec3 = triangle.v2 - triangle.v0;
    let p_vec: Vec3 = ray.direction().cross(vector_v0_to_v2);
    let determinant: f32 = vector_v0_to_v1.dot(p_vec);

    match backface_culling {
        Backfaces::Cull => {
            // if the determinant is negative the triangle is back facing
            // if the determinant is close to 0, the ray misses the triangle
            // This test checks both cases
            if determinant < EPSILON {
                return None;
            }
        }
        Backfaces::Include => {
            // ray and triangle are parallel if det is close to 0
            if determinant.abs() < EPSILON {
                return None;
            }
        }
    }

    let determinant_inverse = 1.0 / determinant;

    let t_vec: Vec3 = ray.origin() - triangle.v0;
    let u = t_vec.dot(p_vec) * determinant_inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q_vec = t_vec.cross(vector_v0_to_v1);
    let v = ray.direction().dot(q_vec) * determinant_inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // The distance between ray origin and intersection is t.
    let t: f32 = vector_v0_to_v2.dot(q_vec) * determinant_inverse;

    // Move along the ray direction from the origin, to find the intersection
    let point_intersection = ray.origin() + ray.direction() * t;
    let triangle_normal = vector_v0_to_v1.cross(vector_v0_to_v2);

    Some(Ray3d::new(point_intersection, triangle_normal))
}

/// Geometric method of computing a ray-triangle intersection
pub fn raycast_geometric(ray: &Ray3d, triangle: &Triangle) -> Option<Ray3d> {
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution
    // compute plane's normal
    let vector_v0_to_v1: Vec3 = triangle.v1 - triangle.v0;
    let vector_v0_to_v2: Vec3 = triangle.v2 - triangle.v0;
    // no need to normalize
    let triangle_normal = vector_v0_to_v1.cross(vector_v0_to_v2); // N

    // Step 1: finding P

    // check if ray and plane are parallel ?
    let n_dot_ray_direction = triangle_normal.dot(ray.direction());
    if n_dot_ray_direction.abs() < EPSILON {
        return None;
    }

    // compute d parameter using equation 2
    let d = triangle_normal.dot(triangle.v0);

    // compute t (equation 3)
    let t = (triangle_normal.dot(ray.origin()) + d) / n_dot_ray_direction;
    // check if the triangle is in behind the ray
    if t < 0.0 {
        return None;
    } // the triangle is behind

    // compute the intersection point using equation 1
    let point_intersection = ray.origin() + t * ray.direction();

    // Step 2: inside-outside test

    // edge 0
    let edge0 = triangle.v1 - triangle.v0;
    let vp0 = point_intersection - triangle.v0;
    let cross = edge0.cross(vp0);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 1
    let edge1 = triangle.v2 - triangle.v1;
    let vp1 = point_intersection - triangle.v1;
    let cross = edge1.cross(vp1);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 2
    let edge2 = triangle.v0 - triangle.v2;
    let vp2 = point_intersection - triangle.v2;
    let cross = edge2.cross(vp2);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side;

    Some(Ray3d::new(point_intersection, triangle_normal))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Triangle vertices to be used in a left-hand coordinate system
    const V0: [f32; 3] = [1.0, -1.0, 2.0];
    const V1: [f32; 3] = [1.0, 2.0, -1.0];
    const V2: [f32; 3] = [1.0, -1.0, -1.0];

    #[test]
    fn raycast_triangle_mt() {
        let triangle = Triangle::from([V0.into(), V1.into(), V2.into()]);
        let ray = Ray3d::new(Vec3::ZERO, Vec3::X);
        let algorithm = RaycastAlgorithm::MollerTrumbore(Backfaces::Include);
        let result = ray_triangle_intersection(&ray, &triangle, algorithm);
        assert_eq!(
            result,
            Some(Ray3d::new([1.0, 0.0, 0.0].into(), [-1.0, 0.0, 0.0].into()))
        );
    }

    #[test]
    fn raycast_triangle_mt_culling() {
        let triangle = Triangle::from([V2.into(), V1.into(), V0.into()]);
        let ray = Ray3d::new(Vec3::ZERO, Vec3::X);
        let algorithm = RaycastAlgorithm::MollerTrumbore(Backfaces::Cull);
        let result = ray_triangle_intersection(&ray, &triangle, algorithm);
        assert_eq!(result, None);
    }

    #[test]
    fn raycast_triangle_geometric() {
        let triangle = Triangle::from([V0.into(), V1.into(), V2.into()]);
        let ray = Ray3d::new(Vec3::ZERO, Vec3::X);
        let algorithm = RaycastAlgorithm::Geometric;
        let result = ray_triangle_intersection(&ray, &triangle, algorithm);
        assert_eq!(
            result,
            Some(Ray3d::new([1.0, 0.0, 0.0].into(), [-1.0, 0.0, 0.0].into()))
        );
    }
}
