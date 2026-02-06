/// Create a `LessVec` with the same syntax as the standard `vec!` macro.
///
/// Supported forms:
/// - `lessvec![]` — empty `LessVec`
/// - `lessvec![a, b, c]` — list of elements
/// - `lessvec![elem; n]` — `n` copies of `elem` (requires `elem: Clone`)
///
/// # Examples
///
/// ```
/// use lessvec::prelude::*;
///
/// let v = lessvec![1, 2, 3];
/// assert_eq!(&*v, &[1, 2, 3]);
///
/// let v2: LessVec<i32> = lessvec![];
/// assert_eq!(v2.len(), 0);
///
/// let v3 = lessvec![5; 4];
/// assert_eq!(&*v3, &[5, 5, 5, 5]);
/// ```
#[macro_export]
macro_rules! lessvec {
    () => {
        $crate::LessVec::new()
    };

    ($elem:expr; $n:expr) => {{
        let mut v = $crate::LessVec::new();
        let count = $n;
        v.reserve(count);
        for _ in 0..count {
            v.push($elem.clone());
        }
        v
    }};

    ($($e:expr),+ $(,)?) => {{
        let mut v = $crate::LessVec::new();
        $( v.push($e); )+
        v
    }};
}
