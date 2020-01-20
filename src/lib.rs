#![feature(const_fn)]

pub mod airspeed;
pub mod wind_estimator;
pub mod errors;
pub mod extensions;
pub mod calculations;


    #[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
