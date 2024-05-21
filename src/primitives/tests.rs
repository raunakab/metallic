use super::*;

#[test]
fn test_point_conversion() {
    const WIDTH: u32 = 100;
    const HEIGHT: u32 = 100;
    let size = PhysicalSize {
        width: WIDTH,
        height: HEIGHT,
    };
    let inputs = [
        (
            Point {
                x: 0.0,
                y: 0.0,
                point_format: PointFormat::Absolute,
            },
            PointFormat::Scaled,
        ),
        (
            Point {
                x: WIDTH as _,
                y: HEIGHT as _,
                point_format: PointFormat::Absolute,
            },
            PointFormat::Scaled,
        ),
        (
            Point {
                x: -1.0,
                y: -1.0,
                point_format: PointFormat::Scaled,
            },
            PointFormat::Absolute,
        ),
        (
            Point {
                x: 1.0,
                y: 1.0,
                point_format: PointFormat::Scaled,
            },
            PointFormat::Absolute,
        ),
    ];
    let expected_outputs = [
        Point {
            x: -1.0,
            y: 1.0,
            point_format: PointFormat::Scaled,
        },
        Point {
            x: 1.0,
            y: -1.0,
            point_format: PointFormat::Scaled,
        },
        Point {
            x: 0.0,
            y: HEIGHT as _,
            point_format: PointFormat::Absolute,
        },
        Point {
            x: WIDTH as _,
            y: 0.0,
            point_format: PointFormat::Absolute,
        },
    ];
    assert_eq!(inputs.len(), expected_outputs.len());
    for ((point, new_point_format), expected_output) in inputs.into_iter().zip(expected_outputs) {
        let actual_output = point.convert(new_point_format, size);
        assert_eq!(actual_output, expected_output);
    }
}
