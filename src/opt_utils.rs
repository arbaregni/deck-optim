use std::fmt;

pub trait OptExt<T: fmt::Display> {
    fn as_option(&self) -> Option<&T>;

    fn display<'a>(&'a self) -> impl fmt::Display + 'a where T: 'a {
        OptDisplay(self.as_option())
    }
}

impl <T> OptExt<T> for Option<T> where T: fmt::Display {
    fn as_option(&self) -> Option<&T> {
        self.as_ref()
    }
}

struct OptDisplay<'a, T>(Option<&'a T>);
impl <'a, T: fmt::Display> fmt::Display for OptDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(inner) => write!(f, "{inner}"),
            None => write!(f, "<none>")
        }
    }
}
