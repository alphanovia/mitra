//! The rows the macros print, and the values as Mitra writes them.

use mitra::{check, check_eq, row, Observe};

#[derive(Debug, PartialEq, Observe)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug, PartialEq, Observe)]
enum Shape {
    Dot,
    Line(Point, Point),
}

#[derive(Debug, PartialEq, Observe)]
struct Boxed<T> {
    held: T,
}

#[derive(Debug, PartialEq, Observe)]
struct Unit;

fn flip(p: Point) -> Point {
    Point { x: p.y, y: p.x }
}
fn even(n: u32) -> bool {
    n.is_multiple_of(2)
}
fn none() -> Option<u8> {
    None
}
mod geo {
    pub fn add(a: i64, b: i64) -> i64 {
        a + b
    }
}

#[test]
fn values_are_written_as_mitra_writes_them() {
    assert_eq!(Point { x: 1, y: -2 }.observe(), "Point { x: 1, y: -2 }");
    assert_eq!(Shape::Dot.observe(), "Dot");
    let line = Shape::Line(Point { x: 0, y: 0 }, Point { x: 1, y: 1 });
    assert_eq!(
        line.observe(),
        "Line(Point { x: 0, y: 0 }, Point { x: 1, y: 1 })"
    );
    assert_eq!(
        Boxed {
            held: vec![Some(1), None]
        }
        .observe(),
        "Boxed { held: [Some(1), None] }"
    );
    assert_eq!(Unit.observe(), "Unit");
    assert_eq!("a\"b".observe(), r#""a\"b""#);
    assert_eq!('c'.observe(), "'c'");
    assert_eq!((1, true).observe(), "(1, true)");
    assert_eq!(Ok::<u8, String>(3).observe(), "Ok(3)");
    assert_eq!([1u8, 2].observe(), "[1, 2]");
}

#[test]
fn a_row_is_the_call_as_written_and_what_it_gave() {
    assert_eq!(
        row("geo :: add", "(1, 2)", &3),
        "mitra: row geo::add (1, 2) -> 3"
    );
    check_eq!(flip(Point { x: 1, y: 2 }), Point { x: 2, y: 1 });
    check_eq!(geo::add(1, 2), 3);
    check_eq!(none(), None);
    check!(even(4));
}

#[test]
#[should_panic(expected = "mitra: row even (3) -> false")]
fn check_says_the_row_when_it_fails() {
    check!(even(3));
}

#[test]
#[should_panic(expected = "mitra: row geo::add (1, 2) -> 3")]
fn check_eq_says_the_row_when_it_fails() {
    check_eq!(geo::add(1, 2), 4);
}
