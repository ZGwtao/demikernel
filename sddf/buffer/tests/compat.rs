use sddf_buffer::DemiBuffer as Ported;
use demikernel::runtime::memory::DemiBuffer as Original;

#[test]
fn heap_buffer_views_match_demikernel() {
    let mut original = Original::from_slice_with_headroom(b"payload", 4).unwrap();
    let mut ported = Ported::from_slice_with_headroom(b"payload", 4).unwrap();

    assert_eq!(original.headroom(), ported.headroom());
    original.prepend(4).unwrap();
    ported.prepend(4).unwrap();
    original[..4].copy_from_slice(b"head");
    ported[..4].copy_from_slice(b"head");
    assert_eq!(&*original, &*ported);

    let original_back = original.split_back(4).unwrap();
    let ported_back = ported.split_back(4).unwrap();
    assert_eq!(&*original, &*ported);
    assert_eq!(&*original_back, &*ported_back);

    original.trim(1).unwrap();
    ported.trim(1).unwrap();
    assert_eq!(&*original, &*ported);

    let mut original = Original::from_slice(b"abcdefgh").unwrap();
    let mut ported = Ported::from_slice(b"abcdefgh").unwrap();
    let original_front = original.split_front(3).unwrap();
    let ported_front = ported.split_front(3).unwrap();
    assert_eq!(&*original, &*ported);
    assert_eq!(&*original_front, &*ported_front);

    let original = Original::from_slice(b"token").unwrap();
    let ported = Ported::from_slice(b"token").unwrap();
    assert_eq!(original.is_direct(), ported.is_direct());
    let original_clone = original.clone();
    let ported_clone = ported.clone();
    assert_eq!(original_clone.is_direct(), ported_clone.is_direct());
    assert_eq!(&*original_clone.into_direct(), &*ported_clone.into_direct());
    let original = unsafe { Original::from_raw(original.into_raw()) };
    let ported = unsafe { Ported::from_raw(ported.into_raw()) };
    assert_eq!(&*original, &*ported);
}
