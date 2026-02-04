pub trait ReadCookie: Clone {
    type UpgradesTo: WriteCookie;

    fn try_upgrade(self) -> Result<Self::UpgradesTo, Self>;
}

pub trait WriteCookie {
    type DowngradesTo: ReadCookie;

    fn downgrade(self) -> Self::DowngradesTo;
}

pub trait CookieJar {
    type ReadToken<'a>: ReadCookie<UpgradesTo = Self::WriteToken<'a>>;
    type WriteToken<'a>: WriteCookie<DowngradesTo = Self::ReadToken<'a>>;

    fn try_read(&self) -> Option<Self::ReadToken<'_>>;
    fn read(&self) -> Self::ReadToken<'_>;
    fn try_write(&self) -> Option<Self::WriteToken<'_>>;
    fn write(&self) -> Self::WriteToken<'_>;
}
