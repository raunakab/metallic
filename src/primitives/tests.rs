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
        AbsPoint(PhysicalPosition { x: 0.0, y: 0.0 }),
        AbsPoint(PhysicalPosition {
            x: WIDTH as _,
            y: HEIGHT as _,
        }),
    ];
    let expected_outputs = [
        ScaledPoint(PhysicalPosition { x: -1.0, y: 1.0 }),
        ScaledPoint(PhysicalPosition { x: 1.0, y: -1.0 }),
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
        ScaledPoint(PhysicalPosition { x: -1.0, y: -1.0 }),
        ScaledPoint(PhysicalPosition { x: 1.0, y: 1.0 }),
    ];
    let expected_outputs = [
        AbsPoint(PhysicalPosition {
            x: 0.0,
            y: HEIGHT as _,
        }),
        AbsPoint(PhysicalPosition {
            x: WIDTH as _,
            y: 0.0,
        }),
    ];
    assert_eq!(inputs.len(), expected_outputs.len());
    for (input, expected_output) in inputs.into_iter().zip(expected_outputs) {
        let actual_output = input.to_abs(SIZE);
        assert_eq!(actual_output, expected_output);
    }
}
