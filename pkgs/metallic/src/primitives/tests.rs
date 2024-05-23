use super::*;

const WIDTH: u32 = 100;
const HEIGHT: u32 = 100;
const SIZE: PhysicalSize<u32> = PhysicalSize {
    width: WIDTH,
    height: HEIGHT,
};

#[test]
fn test_abs_to_scaled_conversion() {
    let inputs = [
        point(0.0, 0.0),
        point(WIDTH as _, HEIGHT as _),
    ];
    let expected_outputs = [
        scaled_point(-1.0, 1.0),
        scaled_point(1.0, -1.0),
    ];
    assert_eq!(inputs.len(), expected_outputs.len());
    for (input, expected_output) in inputs.into_iter().zip(expected_outputs) {
        let actual_output = input.to_scaled(SIZE);
        assert_eq!(actual_output, expected_output);
    }
}

#[test]
fn test_scaled_to_abs_conversion() {
    let inputs = [
        scaled_point(-1.0, -1.0),
        scaled_point(1.0, 1.0),
    ];
    let expected_outputs = [
        point(0.0, HEIGHT as _),
        point(WIDTH as _, 0.0),
    ];
    assert_eq!(inputs.len(), expected_outputs.len());
    for (input, expected_output) in inputs.into_iter().zip(expected_outputs) {
        let actual_output = input.to_abs(SIZE);
        assert_eq!(actual_output, expected_output);
    }
}
