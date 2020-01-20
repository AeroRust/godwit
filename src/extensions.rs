pub trait FloatExt: Sized {
    fn constrain(&self, start: Self, end: Self) -> Self;
}

impl FloatExt for f32 {
    fn constrain(&self, start: Self, end: Self) -> Self {
        if self.is_nan() || start.is_nan() || end.is_nan() {
            std::f32::NAN
        } else if *self >= start && *self <= end {
            *self
        } else if *self > end {
            end
        } else {
            start
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::extensions::FloatExt;

    #[test]
    fn constrain_works() {
        let x = 1.11f32;
        let first = x.constrain( 4.0, 6.0);
        assert_eq!(first, 4.);
        let second = x.constrain(0., 2.);
        assert_eq!(second, 1.11);
        let third = x.constrain(0., 1.111);
        assert_eq!(third, 1.11);
        let fourth= x.constrain(0., 1.1);
        assert_eq!(fourth, 1.1);
    }

    #[test]
    fn constrain_with_nans() {
        let x = 1.;
        let first = x.constrain(0., std::f32::NAN);
        assert!(first.is_nan());
        let second = x.constrain(2., std::f32::NAN);
        assert!(second.is_nan());

        let y = std::f32::NAN;
        assert!(y.constrain(1.0, 2.0).is_nan());
    }
}