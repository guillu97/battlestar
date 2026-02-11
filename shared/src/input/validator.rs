use crate::physics::Input;

/// Validate and clamp input to prevent cheating
pub fn validate_input(input: &mut Input) {
    input.clamp();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_within_range() {
        let mut input = Input::new(0.5, 0.5);
        validate_input(&mut input);
        assert_eq!(input.thrust, 0.5);
        assert_eq!(input.rotate, 0.5);
    }

    #[test]
    fn test_validate_input_clamps_over_max() {
        let mut input = Input::new(2.0, 2.0);
        validate_input(&mut input);
        assert_eq!(input.thrust, 1.0);
        assert_eq!(input.rotate, 1.0);
    }

    #[test]
    fn test_validate_input_clamps_under_min() {
        let mut input = Input::new(-2.0, -2.0);
        validate_input(&mut input);
        assert_eq!(input.thrust, -1.0);
        assert_eq!(input.rotate, -1.0);
    }
}
