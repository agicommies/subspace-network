#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use sp_externalities::ExternalitiesExt;

#[cfg(feature = "std")]
sp_externalities::decl_extension! {
    pub struct OffworkerExt(Box<dyn OffworkerExtension>);
}

#[cfg(feature = "std")]
impl OffworkerExt {
    pub fn new<T: OffworkerExtension>(t: T) -> Self {
        Self(Box::new(t))
    }
}

#[cfg(feature = "std")]
pub trait OffworkerExtension: Send + 'static {
    fn decrypt_weight(&self, encrypted: Vec<u8>) -> Vec<u8>;
}

#[sp_runtime_interface::runtime_interface]
pub trait Offworker {
    fn decrypt_weight(&mut self, encrypted: sp_std::vec::Vec<u8>) -> sp_std::vec::Vec<u8> {
        self.extension::<OffworkerExt>().expect("fooo").decrypt_weight(encrypted)
    }
}
