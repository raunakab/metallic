use super::*;

const LENGTH: u32 = 100;

#[test]
fn test_abs_to_scaled_conversion() {
    let inputs = [
        (0.0, LENGTH),
        ((LENGTH / 2) as _, LENGTH),
        (LENGTH as _, LENGTH),
    ];
    let expected_outputs = [
        -1.0,
        0.0,
        1.0,
    ];
    assert_eq!(inputs.len(), expected_outputs.len());
    for ((x, length), expected_output) in inputs.into_iter().zip(expected_outputs) {
        let actual_output = abs_to_scaled_1d(x, length);
        assert_eq!(actual_output, expected_output);
    }
}
