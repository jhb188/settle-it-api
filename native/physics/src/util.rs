use rapier3d::na::Vector3;

pub fn to_vec3<T>((x, y, z): (T, T, T)) -> Vector3<T> {
    Vector3::new(x, y, z)
}
