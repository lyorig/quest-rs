use std::{
    ops::{Add, Mul, Sub},
    time::Duration,
};

pub trait Animatable:
    Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self> + Copy + Default
{
}

impl<T: Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self> + Copy + Default>
    Animatable for T
{
}

pub enum Anim<T: Animatable> {
    Inactive,
    Active { from: T, to: T, time: f64 },
    Done(T),
}

impl<T: Animatable> Anim<T> {
    pub fn new() -> Self {
        Self::Inactive
    }

    pub fn get(&self) -> T {
        match self {
            Self::Inactive => panic!("Animation inactive"),
            Self::Active { from, to, time } => *from + (*to - *from) * *time,
            Self::Done(t) => *t,
        }
    }

    pub fn update_delta(&mut self, elapsed: Duration) {
        if let Self::Active { from: _, to, time } = self {
            *time += elapsed.as_secs_f64();

            if *time >= 1.0 {
                *self = Self::Done(*to)
            }
        }
    }

    pub fn start(&mut self, from: T, to: T, time: f64) {
        *self = Self::Active { from, to, time };
    }

    pub fn retarget(&mut self, new_to: T) {
        match self {
            Anim::Active {
                from: _,
                to,
                time: _,
            } => *to = new_to,
            _ => panic!("Only active animations can be retargeted"),
        }
    }
}
