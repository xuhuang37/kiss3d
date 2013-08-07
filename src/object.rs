use std::sys;
use std::libc;
use std::num::{One, Zero};
use std::ptr;
use std::cast;
use std::vec;
use glcore::types::GL_VERSION_1_0::*;
use glcore::types::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::consts::GL_VERSION_1_5::*;
use nalgebra::traits::scalar_op::ScalarDiv;
use nalgebra::traits::homogeneous::ToHomogeneous;
use nalgebra::traits::indexable::Indexable;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::norm::Norm;
use nalgebra::adaptors::transform::Transform;
use nalgebra::adaptors::rotmat::Rotmat;
use nalgebra::mat::{Mat3, Mat4};
use nalgebra::vec::Vec3;
use window::Window;

type Transform3d = Transform<Rotmat<Mat3<f64>>, Vec3<f64>>;
type Scale3d     = Mat3<GLfloat>;

pub enum Geometry {
    VerticesNormalsTriangles(~[Vec3<f32>], ~[Vec3<f32>], ~[(GLuint, GLuint, GLuint)]),
    Deleted
}

#[doc(hidden)]
pub struct GeometryIndices {
    priv offset:         uint,
    priv size:           i32,
    priv element_buffer: GLuint,
    priv normal_buffer:  GLuint,
    priv vertex_buffer:  GLuint,
    priv texture_buffer: GLuint
}

impl GeometryIndices {
    #[doc(hidden)]
    pub fn new(offset:         uint,
               size:           i32,
               element_buffer: GLuint,
               normal_buffer:  GLuint,
               vertex_buffer:  GLuint,
               texture_buffer: GLuint) -> GeometryIndices {
        GeometryIndices {
            offset:         offset,
            size:           size,
            element_buffer: element_buffer,
            normal_buffer:  normal_buffer,
            vertex_buffer:  vertex_buffer,
            texture_buffer: texture_buffer
        }
    }
}

/// Structure of all 3d objects on the scene. This is the only interface to manipulate the object
/// position, color, vertices and texture.
pub struct Object {
    priv parent:      @mut Window,
    priv texture:     GLuint,
    priv scale:       Scale3d,
    priv transform:   Transform3d,
    priv color:       Vec3<f32>,
    priv igeometry:   GeometryIndices,
    priv geometry:    Geometry
}

impl Object {
    #[doc(hidden)]
    pub fn new(parent:    @mut Window,
               igeometry: GeometryIndices,
               r:         f32,
               g:         f32,
               b:         f32,
               texture:   GLuint,
               sx:        GLfloat,
               sy:        GLfloat,
               sz:        GLfloat,
               geometry:  Geometry) -> Object {
        Object {
            parent:    parent,
            scale:     Mat3::new(sx, 0.0, 0.0,
                                 0.0, sy, 0.0,
                                 0.0, 0.0, sz),
            transform:   One::one(),
            igeometry:   igeometry,
            geometry:    geometry,
            color:       Vec3::new(r, g, b),
            texture:     texture
        }
    }

    #[doc(hidden)]
    pub fn upload_geometry(&mut self) {
        match self.geometry {
            VerticesNormalsTriangles(ref v, ref n, _) =>
                unsafe {
                    glBindBuffer(GL_ARRAY_BUFFER, self.igeometry.vertex_buffer);
                    glBufferSubData(GL_ARRAY_BUFFER,
                                    0,
                                    (v.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                                    cast::transmute(&v[0]));

                    glBindBuffer(GL_ARRAY_BUFFER, self.igeometry.normal_buffer);
                    glBufferSubData(GL_ARRAY_BUFFER,
                                    0,
                                    (n.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                                    cast::transmute(&n[0]));
                },
                Deleted => { }
        }
    }

    #[doc(hidden)]
    pub fn upload(&self,
                  black_only:                bool,
                  pos_attrib:                u32,
                  normal_attrib:             u32,
                  texture_attrib:            u32,
                  color_location:            i32,
                  transform_location:        i32,
                  scale_location:            i32,
                  normal_transform_location: i32) {

        let formated_transform:  Mat4<f64> = self.transform.to_homogeneous();
        let formated_ntransform: Mat3<f64> = self.transform.submat().submat();

        // we convert the matrix elements and do the transposition at the same time
        let transform_glf = Mat4::new(
            formated_transform.at((0, 0)) as GLfloat,
            formated_transform.at((1, 0)) as GLfloat,
            formated_transform.at((2, 0)) as GLfloat,
            formated_transform.at((3, 0)) as GLfloat,

            formated_transform.at((0, 1)) as GLfloat,
            formated_transform.at((1, 1)) as GLfloat,
            formated_transform.at((2, 1)) as GLfloat,
            formated_transform.at((3, 1)) as GLfloat,

            formated_transform.at((0, 2)) as GLfloat,
            formated_transform.at((1, 2)) as GLfloat,
            formated_transform.at((2, 2)) as GLfloat,
            formated_transform.at((3, 2)) as GLfloat,

            formated_transform.at((0, 3)) as GLfloat,
            formated_transform.at((1, 3)) as GLfloat,
            formated_transform.at((2, 3)) as GLfloat,
            formated_transform.at((3, 3)) as GLfloat
            );

        let ntransform_glf = Mat3::new(
            formated_ntransform.at((0, 0)) as GLfloat,
            formated_ntransform.at((1, 0)) as GLfloat,
            formated_ntransform.at((2, 0)) as GLfloat,
            formated_ntransform.at((0, 1)) as GLfloat,
            formated_ntransform.at((1, 1)) as GLfloat,
            formated_ntransform.at((2, 1)) as GLfloat,
            formated_ntransform.at((0, 2)) as GLfloat,
            formated_ntransform.at((1, 2)) as GLfloat,
            formated_ntransform.at((2, 2)) as GLfloat
            );

        unsafe {
            glUniformMatrix4fv(transform_location,
                               1,
                               GL_FALSE,
                               cast::transmute(&transform_glf));

            glUniformMatrix3fv(normal_transform_location,
                               1,
                               GL_FALSE,
                               cast::transmute(&ntransform_glf));

            glUniformMatrix3fv(scale_location, 1, GL_FALSE, cast::transmute(&self.scale));

            if black_only {
                glUniform3f(color_location, 0.0, 0.0, 0.0);
            }
            else {
                glUniform3f(color_location, self.color.x, self.color.y, self.color.z);
            }

            // FIXME: we should not switch the buffers if the last drawn shape uses the same.
            glBindBuffer(GL_ARRAY_BUFFER, self.igeometry.vertex_buffer);
            glVertexAttribPointer(pos_attrib,
                                  3,
                                  GL_FLOAT,
                                  GL_FALSE,
                                  3 * sys::size_of::<GLfloat>() as GLsizei,
                                  ptr::null());

            glBindBuffer(GL_ARRAY_BUFFER, self.igeometry.normal_buffer);
            glVertexAttribPointer(normal_attrib,
                                  3,
                                  GL_FLOAT,
                                  GL_FALSE,
                                  3 * sys::size_of::<GLfloat>() as GLsizei,
                                  ptr::null());

            glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, self.igeometry.element_buffer);

            glBindTexture(GL_TEXTURE_2D, self.texture);

            glBindBuffer(GL_ARRAY_BUFFER, self.igeometry.texture_buffer);

            glVertexAttribPointer(texture_attrib,
                                  2,
                                  GL_FLOAT,
                                  GL_FALSE,
                                  2 * sys::size_of::<GLfloat>() as GLsizei,
                                  ptr::null());

            glDrawElements(GL_TRIANGLES,
                           self.igeometry.size,
                           GL_UNSIGNED_INT,
                           (self.igeometry.offset * sys::size_of::<GLuint>()) as *libc::c_void);
        }
    }

    /// The 3d transformation of the object. This is an isometric transformation (i-e no scaling).
    pub fn transformation<'r>(&'r mut self) -> &'r mut Transform3d {
        &mut self.transform
    }

    /// The object geometry. Some geometries might not be
    /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    pub fn geometry<'r>(&'r self) -> &'r Geometry {
        &'r self.geometry
    }

    /// Applies a user-defined callback on the object geometry. Some geometries might not be
    /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    ///
    /// # Arguments
    ///   * `f` - A user-defined callback called on the object geometry. If it returns `true`, the
    ///   geometry will be updated on graphics memory too. Otherwise, the modification will not have
    ///   any effect on the 3d display.
    pub fn modify_geometry(&mut self,
                           f: &fn(vertices:  &mut ~[Vec3<f32>],
                           normals:   &mut ~[Vec3<f32>],
                           triangles: &mut ~[(GLuint, GLuint, GLuint)]) -> bool) {
        if match self.geometry {
            VerticesNormalsTriangles(ref mut v, ref mut n, ref mut t) => f(v, n, t),
            Deleted => false
        } {
            self.upload_geometry()
        }
    }

    /// Applies a user-defined callback on the object vertices. Some geometries might not be
    /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    ///
    /// # Arguments
    ///   * `f` - A user-defined callback called on the object vertice. The normals are automatically
    ///   recomputed. If it returns `true`, the the geometry will be updated on graphics memory too.
    ///   Otherwise, the modifications will not have any effect on the 3d display.
    pub fn modify_vertices(&mut self, f: &fn(&mut ~[Vec3<f32>]) -> bool) {
        let (update, normals) = match self.geometry {
            VerticesNormalsTriangles(ref mut v, _, _) => (f(v), true),
            Deleted => (false, false)
        };

        if normals {
            self.recompute_normals()
        }

        if update {
            self.upload_geometry()
        }
    }

    fn recompute_normals(&mut self) {
        match self.geometry {
            VerticesNormalsTriangles(ref vs, ref mut ns, ref ts) => {
                let mut divisor = vec::from_elem(vs.len(), 0f32);

                // ... and compute the mean
                for n in ns.mut_iter() {
                    *n = Zero::zero()
                }

                // accumulate normals...
                for &(v1, v2, v3) in ts.iter() {
                    let edge1 = vs[v2] - vs[v1];
                    let edge2 = vs[v3] - vs[v1];
                    let normal = edge1.cross(&edge2).normalized();

                    ns[v1] = ns[v1] + normal;
                    ns[v2] = ns[v2] + normal;
                    ns[v3] = ns[v3] + normal;

                    divisor[v1] = divisor[v1] + 1.0;
                    divisor[v2] = divisor[v2] + 1.0;
                    divisor[v3] = divisor[v3] + 1.0;
                }

                // ... and compute the mean
                for (n, divisor) in ns.mut_iter().zip(divisor.iter()) {
                    n.scalar_div_inplace(divisor)
                }
            },
            Deleted => { }
        }
    }

    /// Sets the color of the object. Colors components must be on the range `[0.0, 1.0]`.
    pub fn set_color(@mut self, r: f32, g: f32, b: f32) -> @mut Object {
        self.color.x = r;
        self.color.y = g;
        self.color.z = b;

        self
    }

    /// Sets the texture of the object.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    pub fn set_texture(@mut self, path: ~str) -> @mut Object {
        self.texture = self.parent.add_texture(path);

        self
    }
}
