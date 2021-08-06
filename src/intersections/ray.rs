use num_traits::Inv;
use std::ops::{Mul, Sub};

// A 2-dimensional vector.
struct Vec2<N> {
    pub x: N,
    pub y: N,
}

impl<T> Inv for Vec2<T>
where
    T: Inv<Output = T>,
{
    type Output = Vec2<T>;

    fn inv(self) -> Self::Output {
        Self {
            x: self.x.inv(),
            y: self.y.inv(),
        }
    }
}

/// A ray.
pub struct Ray<T>
where
    T: Inv<Output = T> + Clone,
{
    origin: T,
    direction: T,
    inv_direction: T,
}

impl<T> Ray<T>
where
    T: Inv<Output = T> + Clone,
{
    pub fn new(origin: T, direction: T) -> Self {
        let inv = direction.clone().inv();
        Self {
            origin,
            direction,
            inv_direction: inv,
        }
    }
}

impl<T> Clone for Vec2<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

/// A 2-dimensional box.
struct Box2<N>
where
    N: MinMax,
{
    pub min: Vec2<N>,
    pub max: Vec2<N>,
}

/// Trait for box-ray intersections.
trait RayIntersection<T>
where
    T: Inv<Output = T> + Clone,
{
    fn intersects(&self, ray: &Ray<T>) -> bool;
}

impl<T> RayIntersection<Vec2<T>> for Box2<T>
where
    T: Sub<Output = T>
        + Mul<Output = T>
        + Inv<Output = T>
        + MinMax
        + Clone
        + PartialOrd<T>
        + Default,
{
    fn intersects(&self, ray: &Ray<Vec2<T>>) -> bool {
        let diff_min_x = self.min.x.clone() - ray.origin.x.clone();
        let diff_max_x = self.max.x.clone() - ray.origin.x.clone();
        let diff_min_y = self.min.y.clone() - ray.origin.y.clone();
        let diff_max_y = self.max.y.clone() - ray.origin.y.clone();

        let tx1 = diff_min_x * ray.inv_direction.x.clone();
        let tx2 = diff_max_x * ray.inv_direction.x.clone();
        let ty1 = diff_min_y * ray.inv_direction.y.clone();
        let ty2 = diff_max_y * ray.inv_direction.y.clone();

        let tmin_x = tx1.clone().min_(tx2.clone());
        let tmax_x = tx1.max_(tx2);
        let tmin_y = ty1.clone().min_(ty2.clone());
        let tmax_y = ty1.max_(ty2);

        let tmin = tmin_x.max_(tmin_y);
        let tmax = tmax_x.min_(tmax_y);

        tmax >= tmin && tmax >= T::default()
    }
}

trait MinMax: PartialOrd + Sized {
    fn min_(self, rhs: Self) -> Self {
        if self < rhs {
            self
        } else {
            rhs
        }
    }

    fn max_(self, rhs: Self) -> Self {
        if self > rhs {
            self
        } else {
            rhs
        }
    }
}

impl MinMax for f32 {
    fn min_(self, rhs: Self) -> Self {
        self.min(rhs)
    }

    fn max_(self, rhs: Self) -> Self {
        self.max(rhs)
    }
}

impl MinMax for f64 {
    fn min_(self, rhs: Self) -> Self {
        self.min(rhs)
    }

    fn max_(self, rhs: Self) -> Self {
        self.max(rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn box_in_front_works() {
        let box2d = Box2 {
            min: Vec2 { x: -1., y: -1. },
            max: Vec2 { x: 1., y: 1. },
        };

        // The ray originates "in front of" the box and points towards it.
        // Therefore, we must observe an intersection.
        let ray = Ray::new(Vec2 { x: -10., y: 0. }, Vec2 { x: 1., y: 0.1 });
        assert!(box2d.intersects(&ray));
    }

    #[test]
    fn box_behind_works() {
        let box2d = Box2 {
            min: Vec2 { x: -1., y: -1. },
            max: Vec2 { x: 1., y: 1. },
        };

        // The ray originates "behind" the box and points away from it.
        // Therefore, we must not observe an intersection.
        let ray = Ray::new(Vec2 { x: 10., y: 0. }, Vec2 { x: 1., y: 0. });
        assert!(!box2d.intersects(&ray));
    }
}
