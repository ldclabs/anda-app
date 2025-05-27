use core::{
    fmt::{Debug, Display},
    ops::Deref,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut rng = rand::rng();
    let mut bytes = [0u8; N];
    rng.fill_bytes(&mut bytes);
    bytes
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct SensitiveData<T>(pub T);

impl<T> Default for SensitiveData<T>
where
    T: Default,
{
    fn default() -> Self {
        SensitiveData(T::default())
    }
}

impl<T> Display for SensitiveData<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Debug for SensitiveData<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Vec<char> = self.0.to_string().chars().collect();
        match s.len() {
            v if v >= 11 => write!(
                f,
                "{}{}{}**{}{}{}",
                s[0],
                s[1],
                s[2],
                s[s.len() - 3],
                s[s.len() - 2],
                s[s.len() - 1]
            ),
            v if v >= 5 => write!(f, "{}**{}", s[0], s[s.len() - 1]),
            _ => write!(f, "**"),
        }
    }
}

impl<T> AsRef<T> for SensitiveData<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for SensitiveData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for SensitiveData<T> {
    fn from(value: T) -> Self {
        SensitiveData(value)
    }
}
