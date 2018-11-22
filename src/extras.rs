pub mod option {
    // replace with std version once it hits stable
    pub fn transpose<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
        match x {
            Some(Ok(x)) => Ok(Some(x)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}
