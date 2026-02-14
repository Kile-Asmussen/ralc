pub mod cell;
pub mod parking_lot;

/// A cookie representing permission to take shared references to a resource.
pub trait ReadCookie: Clone {
    type UpgradesTo: WriteCookie;

    /// Attempt to upgrade this cookie into permission to take exclusive access.
    fn try_upgrade(self) -> Result<Self::UpgradesTo, Self>;
}

/// A cookie representing permission to take exclusive reference to a resource.
pub trait WriteCookie {
    type DowngradesTo: ReadCookie;

    /// Relinquish the right to have exclusive reference, but retain permission to take shared referencess.
    fn downgrade(self) -> Self::DowngradesTo;
}

pub trait CookieJar: Default {
    type ReadToken<'a>: ReadCookie<UpgradesTo = Self::WriteToken<'a>>;
    type WriteToken<'a>: WriteCookie<DowngradesTo = Self::ReadToken<'a>>;

    /// Provide an approximate count of the number of readers of this lock,
    /// returning u32::MAX if a write lock exists. Only available for testing.
    #[cfg(test)]
    fn count(&self) -> u32;

    /// Attempt to acquire a read-cookie for this lock representing permission to
    /// take shared references to the guarded resource. Return `None` immediately
    /// if this is not possible, due to an ongoing write operation.
    fn try_read(&self) -> Option<Self::ReadToken<'_>>;

    /// Acquire a read-cookie for this lock representing permission to
    /// take shared references to the guarded resource. Return `None` if this
    /// is not possible, such as waiting operations not being available.
    fn read(&self) -> Option<Self::ReadToken<'_>> {
        self.try_read()
    }

    /// Attempt to acquire a write-cookie for this lock representing permission to
    /// take exclusive reference to the guarded resource. Return immediately
    /// if this is not possible, due to another ongoing operation.
    fn try_write(&self) -> Option<Self::WriteToken<'_>>;

    /// Acquire a rite-cookie for this lock representing permission to
    /// take exclusive reference to the guarded resource. Return `None` if this
    /// is not possible, such as waiting operations not being available.
    fn write(&self) -> Option<Self::WriteToken<'_>> {
        self.try_write()
    }
}
