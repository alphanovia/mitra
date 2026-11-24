//! Tests that write what they saw. `check_eq!(f(a, b), want)` calls `f`,
//! prints the row it saw, `mitra: row f (a, b) -> got`, then asserts
//! `got == want`. `check!(f(a, b))` does the same for a function that
//! gives `bool`, and asserts it gave `true`.
//!
//! `mitra explain` runs `cargo test --no-fail-fast -- --show-output`,
//! reads each row under its test's name, and finds the model's function
//! through the tie on the Rust one (`// mitra: implements a::f`): a row
//! is a measurement of what the system gave, whether the test passed or
//! not; the test's own verdict is the assert. A row through no tie is
//! R007.
//!
//! Values are written as Mitra writes them, by `Observe`: derive it for
//! your types (`#[cfg_attr(test, derive(mitra::Observe))]`). The call is
//! by path, `f(…)`, `geo::flip(…)` or `Heater::decide(h, 18)` for a
//! method, its receiver first, as the model's `self`.

pub use mitra_derive::Observe;

/// A value written as Mitra writes it: `6`, `true`, `"text"`, `[1, 2]`,
/// `Some(3)`, `(1, true)`, `Point { x: 1, y: 2 }`.
pub trait Observe {
    fn observe(&self) -> String;
}

macro_rules! shown_by_display {
    ($($t:ty),*) => {$(
        impl Observe for $t {
            fn observe(&self) -> String {
                self.to_string()
            }
        }
    )*};
}
shown_by_display!(bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl Observe for char {
    fn observe(&self) -> String {
        format!("{self:?}")
    }
}
impl Observe for str {
    fn observe(&self) -> String {
        format!("{self:?}")
    }
}
impl Observe for String {
    fn observe(&self) -> String {
        self.as_str().observe()
    }
}
impl<T: Observe + ?Sized> Observe for &T {
    fn observe(&self) -> String {
        (**self).observe()
    }
}
impl<T: Observe + ?Sized> Observe for &mut T {
    fn observe(&self) -> String {
        (**self).observe()
    }
}
impl<T: Observe + ?Sized> Observe for Box<T> {
    fn observe(&self) -> String {
        (**self).observe()
    }
}
impl<T: Observe> Observe for [T] {
    fn observe(&self) -> String {
        let holds: Vec<String> = self.iter().map(Observe::observe).collect();
        format!("[{}]", holds.join(", "))
    }
}
impl<T: Observe, const N: usize> Observe for [T; N] {
    fn observe(&self) -> String {
        self.as_slice().observe()
    }
}
impl<T: Observe> Observe for Vec<T> {
    fn observe(&self) -> String {
        self.as_slice().observe()
    }
}
impl<T: Observe> Observe for Option<T> {
    fn observe(&self) -> String {
        match self {
            Some(v) => format!("Some({})", v.observe()),
            None => "None".to_string(),
        }
    }
}
impl<T: Observe, E: Observe> Observe for Result<T, E> {
    fn observe(&self) -> String {
        match self {
            Ok(v) => format!("Ok({})", v.observe()),
            Err(e) => format!("Err({})", e.observe()),
        }
    }
}

/// What a function was given, `(a, b)`, as a row writes it. Up to
/// eight inputs.
pub trait Inputs {
    fn inputs(&self) -> String;
}

/// The function called on what it was given.
pub trait Apply<F, R> {
    fn apply(self, f: F) -> R;
}

macro_rules! inputs {
    ($(($($a:ident $i:tt),*))*) => {$(
        impl<$($a: Observe),*> Inputs for ($($a,)*) {
            fn inputs(&self) -> String {
                let shown: Vec<String> = vec![$(self.$i.observe()),*];
                format!("({})", shown.join(", "))
            }
        }
        impl<$($a,)* R, F: FnOnce($($a),*) -> R> Apply<F, R> for ($($a,)*) {
            #[allow(clippy::unused_unit)]
            fn apply(self, f: F) -> R {
                f($(self.$i),*)
            }
        }
        // Mitra has no tuple of one: a pair and up, as outputs.
        inputs!(@tuple $($a $i),*);
    )*};
    (@tuple) => {};
    (@tuple $a:ident $i:tt) => {};
    (@tuple $($a:ident $i:tt),*) => {
        impl<$($a: Observe),*> Observe for ($($a,)*) {
            fn observe(&self) -> String {
                let shown: Vec<String> = vec![$(self.$i.observe()),*];
                format!("({})", shown.join(", "))
            }
        }
    };
}
inputs! {
    ()
    (A 0)
    (A 0, B 1)
    (A 0, B 1, C 2)
    (A 0, B 1, C 2, D 3)
    (A 0, B 1, C 2, D 3, E 4)
    (A 0, B 1, C 2, D 3, E 4, G 5)
    (A 0, B 1, C 2, D 3, E 4, G 5, H 6)
    (A 0, B 1, C 2, D 3, E 4, G 5, H 6, I 7)
}

/// The line `mitra explain` reads: `mitra: row f (a, b) -> got`. The
/// path as written in the call, spaces taken out.
#[doc(hidden)]
pub fn row(path: &str, inputs: &str, got: &dyn Observe) -> String {
    let path: String = path.split_whitespace().collect();
    let line = format!("mitra: row {path} {inputs} -> {}", got.observe());
    println!("{line}");
    line
}

/// `check_eq!(f(a, b), want)`: call `f`, print the row seen, then assert
/// it gave `want`.
#[macro_export]
macro_rules! check_eq {
    ($($f:ident)::+ ( $($a:expr),* $(,)? ), $want:expr $(,)?) => {{
        let args = ($($a,)*);
        let inputs = $crate::Inputs::inputs(&args);
        let got = $crate::Apply::apply(args, $($f)::+);
        let line = $crate::row(stringify!($($f)::+), &inputs, &got);
        assert_eq!(got, $want, "{}", line);
    }};
}

/// `check!(f(a, b))`: call `f`, which gives `bool`, print the row seen,
/// then assert it gave `true`.
#[macro_export]
macro_rules! check {
    ($($f:ident)::+ ( $($a:expr),* $(,)? ) $(,)?) => {{
        let args = ($($a,)*);
        let inputs = $crate::Inputs::inputs(&args);
        let got: bool = $crate::Apply::apply(args, $($f)::+);
        let line = $crate::row(stringify!($($f)::+), &inputs, &got);
        assert!(got, "{}", line);
    }};
}
