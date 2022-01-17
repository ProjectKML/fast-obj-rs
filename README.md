# fast_obj_rs
Low-level rust bindings for the blazing obj parser fast_obj written in C

The current version is an early version, only the ffi functions are exposed!

Example:
```Rust
use fast_obj_rs::ffi;

fn main() {
    unsafe {
        let p = ffi::fast_obj_read(b"dragon.obj\0".as_ptr() as *const i8);
        if p == std::ptr::null_mut() {
            panic!("Failed to load mesh");
        }

        assert_eq!((*p).face_count, 100000);

        ffi::fast_obj_destroy(p);
    }
}
```
