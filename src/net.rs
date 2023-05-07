use fundsp::prelude::*;
use janetrs::{janet_abstract::*, IsJanetAbstract, Janet};
use std::{ffi::*, mem::ManuallyDrop, ops::Deref};

pub struct Net(pub ManuallyDrop<Net64>);

extern "C" fn net_gc(data: *mut c_void, _len: usize) -> c_int {
    unsafe {
        let mut a = JanetAbstract::from_raw(data);
        let net: &mut Net = a.get_mut_unchecked();
        // We don't own the value so we can't use ManuallyDrop::into_inner.
        ManuallyDrop::drop(&mut net.0);
    };

    0
}

const NET_TYPE: JanetAbstractType = JanetAbstractType {
    name: "net\0" as *const str as *const std::ffi::c_char,
    gc: Some(net_gc),
    gcmark: None,
    get: None,
    put: None,
    marshal: None,
    unmarshal: None,
    tostring: None,
    compare: None,
    hash: None,
    next: None,
    call: None,
    length: None,
    bytes: None,
};

impl IsJanetAbstract for Net {
    const SIZE: usize = std::mem::size_of::<Self>();

    #[inline]
    fn type_info() -> &'static JanetAbstractType {
        &NET_TYPE
    }
}

impl Deref for Net {
    type Target = Net64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<An<T>> for Net
where
    T: AudioNode<Sample = f64> + Send + Sync + 'static,
{
    fn from(an: An<T>) -> Self {
        Net(ManuallyDrop::new(Net64::wrap(Box::new(an))))
    }
}

// impl Drop for Net {
//     fn drop(&mut self) {
//         println!("drop net");
//     }
// }

impl From<Net> for Janet {
    fn from(net: Net) -> Self {
        Janet::j_abstract(JanetAbstract::new(net))
    }
}
