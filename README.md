# fast-obj-rs
Rust bindings for the blazing obj parser fast_obj written in C

Example:
```Rust
use fast_obj_rs::Mesh;

fn main() {
    let mesh = Mesh::new("dragon.obj").unwrap();

    mesh.positions().iter().for_each(|position| {
        println!("Position: {}", position);
    });

    mesh.texcoords().iter().for_each(|tex_coord| {
        println!("Tex Coord: {}", tex_coord);
    });

    mesh.normals().iter().for_each(|normal| {
        println!("Normal: {}", normal);
    });
}
```
