extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{UnitQuaternion, Vector3};

fn main() {
    let mut window = Window::new("Kiss3d: wireframe");
    let mut c = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);
    c.set_points_size(10.0);
    c.set_lines_width(1.0);
    c.set_surface_rendering_activation(false);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }
}
