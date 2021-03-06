// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::{UnknownUnit, Radians};
use approxeq::ApproxEq;
use trig::Trig;
use point::{TypedPoint2D, TypedPoint3D, TypedPoint4D};
use matrix2d::TypedMatrix2D;
use scale_factor::ScaleFactor;
use num::{One, Zero};
use std::ops::{Add, Mul, Sub, Div, Neg};
use std::marker::PhantomData;
use std::fmt;

define_matrix! {
    /// A 4 by 4 matrix stored in row-major order in memory, useful to represent
    /// 3d transformations.
    ///
    /// Matrices can be parametrized over the source and destination units, to describe a
    /// transformation from a space to another.
    /// For example, TypedMatrix4D<f32, WordSpace, ScreenSpace>::transform_point4d
    /// takes a TypedPoint4D<f32, WordSpace> and returns a TypedPoint4D<f32, ScreenSpace>.
    ///
    /// Matrices expose a set of convenience methods for pre- and post-transformations.
    /// A pre-transformation corresponds to adding an operation that is applied before
    /// the rest of the transformation, while a post-transformation adds an operation
    /// that is appled after.
    pub struct TypedMatrix4D<T, Src, Dst> {
        pub m11: T, pub m12: T, pub m13: T, pub m14: T,
        pub m21: T, pub m22: T, pub m23: T, pub m24: T,
        pub m31: T, pub m32: T, pub m33: T, pub m34: T,
        pub m41: T, pub m42: T, pub m43: T, pub m44: T,
    }
}

/// The default 4d matrix type with no units.
pub type Matrix4D<T> = TypedMatrix4D<T, UnknownUnit, UnknownUnit>;

impl<T, Src, Dst> TypedMatrix4D<T, Src, Dst> {
    /// Create a matrix specifying its components in row-major order.
    ///
    /// For example, the translation terms m41, m42, m43 on the last row with the
    /// row-major convention) are the 13rd, 14th and 15th parameters.
    #[inline]
    pub fn row_major(
            m11: T, m12: T, m13: T, m14: T,
            m21: T, m22: T, m23: T, m24: T,
            m31: T, m32: T, m33: T, m34: T,
            m41: T, m42: T, m43: T, m44: T)
         -> TypedMatrix4D<T, Src, Dst> {
        TypedMatrix4D {
            m11: m11, m12: m12, m13: m13, m14: m14,
            m21: m21, m22: m22, m23: m23, m24: m24,
            m31: m31, m32: m32, m33: m33, m34: m34,
            m41: m41, m42: m42, m43: m43, m44: m44,
            _unit: PhantomData,
        }
    }

    /// Create a matrix specifying its components in column-major order.
    ///
    /// For example, the translation terms m41, m42, m43 on the last column with the
    /// column-major convention) are the 4th, 8th and 12nd parameters.
    #[inline]
    pub fn column_major(
            m11: T, m21: T, m31: T, m41: T,
            m12: T, m22: T, m32: T, m42: T,
            m13: T, m23: T, m33: T, m43: T,
            m14: T, m24: T, m34: T, m44: T)
         -> TypedMatrix4D<T, Src, Dst> {
        TypedMatrix4D {
            m11: m11, m12: m12, m13: m13, m14: m14,
            m21: m21, m22: m22, m23: m23, m24: m24,
            m31: m31, m32: m32, m33: m33, m34: m34,
            m41: m41, m42: m42, m43: m43, m44: m44,
            _unit: PhantomData,
        }
    }
}

impl <T, Src, Dst> TypedMatrix4D<T, Src, Dst>
where T: Copy + Clone +
         Add<T, Output=T> +
         Sub<T, Output=T> +
         Mul<T, Output=T> +
         Div<T, Output=T> +
         Neg<Output=T> +
         ApproxEq<T> +
         PartialOrd +
         Trig +
         One + Zero {

    /// Create a 4 by 4 matrix representing a 2d transformation, specifying its components
    /// in row-major order.
    #[inline]
    pub fn row_major_2d(m11: T, m12: T, m21: T, m22: T, m41: T, m42: T) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        TypedMatrix4D::row_major(
            m11, m12, _0, _0,
            m21, m22, _0, _0,
             _0,  _0, _1, _0,
            m41, m42, _0, _1
       )
    }

    /// Create an orthogonal projection matrix.
    pub fn ortho(left: T, right: T,
                 bottom: T, top: T,
                 near: T, far: T) -> TypedMatrix4D<T, Src, Dst> {
        let tx = -((right + left) / (right - left));
        let ty = -((top + bottom) / (top - bottom));
        let tz = -((far + near) / (far - near));

        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        let _2 = _1 + _1;
        TypedMatrix4D::row_major(
            _2 / (right - left), _0                 , _0                , _0,
            _0                 , _2 / (top - bottom), _0                , _0,
            _0                 , _0                 , -_2 / (far - near), _0,
            tx                 , ty                 , tz                , _1
        )
    }

    #[inline]
    pub fn identity() -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        TypedMatrix4D::row_major(
            _1, _0, _0, _0,
            _0, _1, _0, _0,
            _0, _0, _1, _0,
            _0, _0, _0, _1
        )
    }

    /// Returns true if this matrix can be represented with a TypedMatrix2D.
    ///
    /// See https://drafts.csswg.org/css-transforms/#2d-matrix
    #[inline]
    pub fn is_2d(&self) -> bool {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        self.m31 == _0 && self.m32 == _0 &&
        self.m13 == _0 && self.m23 == _0 &&
        self.m43 == _0 && self.m14 == _0 &&
        self.m24 == _0 && self.m34 == _0 &&
        self.m33 == _1 && self.m44 == _1
    }

    /// Create a 2D matrix picking the relevent terms from this matrix.
    ///
    /// This method assumes that self represents a 2d transformation, callers
    /// should check that self.is_2d() returns true beforehand.
    pub fn to_2d(&self) -> TypedMatrix2D<T, Src, Dst> {
        TypedMatrix2D::row_major(
            self.m11, self.m12,
            self.m21, self.m22,
            self.m41, self.m42
        )
    }

    pub fn approx_eq(&self, other: &TypedMatrix4D<T, Src, Dst>) -> bool {
        self.m11.approx_eq(&other.m11) && self.m12.approx_eq(&other.m12) &&
        self.m13.approx_eq(&other.m13) && self.m14.approx_eq(&other.m14) &&
        self.m21.approx_eq(&other.m21) && self.m22.approx_eq(&other.m22) &&
        self.m23.approx_eq(&other.m23) && self.m24.approx_eq(&other.m24) &&
        self.m31.approx_eq(&other.m31) && self.m32.approx_eq(&other.m32) &&
        self.m33.approx_eq(&other.m33) && self.m34.approx_eq(&other.m34) &&
        self.m41.approx_eq(&other.m41) && self.m42.approx_eq(&other.m42) &&
        self.m43.approx_eq(&other.m43) && self.m44.approx_eq(&other.m44)
    }

    /// Returns the same matrix with a different destination unit.
    #[inline]
    pub fn with_destination<NewDst>(&self) -> TypedMatrix4D<T, Src, NewDst> {
        TypedMatrix4D::row_major(
            self.m11, self.m12, self.m13, self.m14,
            self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34,
            self.m41, self.m42, self.m43, self.m44,
        )
    }

    /// Returns the same matrix with a different source unit.
    #[inline]
    pub fn with_source<NewSrc>(&self) -> TypedMatrix4D<T, NewSrc, Dst> {
        TypedMatrix4D::row_major(
            self.m11, self.m12, self.m13, self.m14,
            self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34,
            self.m41, self.m42, self.m43, self.m44,
        )
    }

    /// Returns the multiplication of the two matrices such that mat's transformation
    /// applies after self's transformation.
    pub fn post_mul<NewDst>(&self, mat: &TypedMatrix4D<T, Dst, NewDst>) -> TypedMatrix4D<T, Src, NewDst> {
        TypedMatrix4D::row_major(
            self.m11 * mat.m11  +  self.m12 * mat.m21  +  self.m13 * mat.m31  +  self.m14 * mat.m41,
            self.m11 * mat.m12  +  self.m12 * mat.m22  +  self.m13 * mat.m32  +  self.m14 * mat.m42,
            self.m11 * mat.m13  +  self.m12 * mat.m23  +  self.m13 * mat.m33  +  self.m14 * mat.m43,
            self.m11 * mat.m14  +  self.m12 * mat.m24  +  self.m13 * mat.m34  +  self.m14 * mat.m44,
            self.m21 * mat.m11  +  self.m22 * mat.m21  +  self.m23 * mat.m31  +  self.m24 * mat.m41,
            self.m21 * mat.m12  +  self.m22 * mat.m22  +  self.m23 * mat.m32  +  self.m24 * mat.m42,
            self.m21 * mat.m13  +  self.m22 * mat.m23  +  self.m23 * mat.m33  +  self.m24 * mat.m43,
            self.m21 * mat.m14  +  self.m22 * mat.m24  +  self.m23 * mat.m34  +  self.m24 * mat.m44,
            self.m31 * mat.m11  +  self.m32 * mat.m21  +  self.m33 * mat.m31  +  self.m34 * mat.m41,
            self.m31 * mat.m12  +  self.m32 * mat.m22  +  self.m33 * mat.m32  +  self.m34 * mat.m42,
            self.m31 * mat.m13  +  self.m32 * mat.m23  +  self.m33 * mat.m33  +  self.m34 * mat.m43,
            self.m31 * mat.m14  +  self.m32 * mat.m24  +  self.m33 * mat.m34  +  self.m34 * mat.m44,
            self.m41 * mat.m11  +  self.m42 * mat.m21  +  self.m43 * mat.m31  +  self.m44 * mat.m41,
            self.m41 * mat.m12  +  self.m42 * mat.m22  +  self.m43 * mat.m32  +  self.m44 * mat.m42,
            self.m41 * mat.m13  +  self.m42 * mat.m23  +  self.m43 * mat.m33  +  self.m44 * mat.m43,
            self.m41 * mat.m14  +  self.m42 * mat.m24  +  self.m43 * mat.m34  +  self.m44 * mat.m44,
        )
    }

    /// Returns the multiplication of the two matrices such that mat's transformation
    /// applies before self's transformation.
    pub fn pre_mul<NewSrc>(&self, mat: &TypedMatrix4D<T, NewSrc, Src>) -> TypedMatrix4D<T, NewSrc, Dst> {
        mat.post_mul(self)
    }

    /// Returns the inverse matrix if possible.
    pub fn inverse(&self) -> Option<TypedMatrix4D<T, Dst, Src>> {
        let det = self.determinant();

        if det == Zero::zero() {
            return None;
        }

        // todo(gw): this could be made faster by special casing
        // for simpler matrix types.
        let m = TypedMatrix4D::row_major(
            self.m23*self.m34*self.m42 - self.m24*self.m33*self.m42 +
            self.m24*self.m32*self.m43 - self.m22*self.m34*self.m43 -
            self.m23*self.m32*self.m44 + self.m22*self.m33*self.m44,

            self.m14*self.m33*self.m42 - self.m13*self.m34*self.m42 -
            self.m14*self.m32*self.m43 + self.m12*self.m34*self.m43 +
            self.m13*self.m32*self.m44 - self.m12*self.m33*self.m44,

            self.m13*self.m24*self.m42 - self.m14*self.m23*self.m42 +
            self.m14*self.m22*self.m43 - self.m12*self.m24*self.m43 -
            self.m13*self.m22*self.m44 + self.m12*self.m23*self.m44,

            self.m14*self.m23*self.m32 - self.m13*self.m24*self.m32 -
            self.m14*self.m22*self.m33 + self.m12*self.m24*self.m33 +
            self.m13*self.m22*self.m34 - self.m12*self.m23*self.m34,

            self.m24*self.m33*self.m41 - self.m23*self.m34*self.m41 -
            self.m24*self.m31*self.m43 + self.m21*self.m34*self.m43 +
            self.m23*self.m31*self.m44 - self.m21*self.m33*self.m44,

            self.m13*self.m34*self.m41 - self.m14*self.m33*self.m41 +
            self.m14*self.m31*self.m43 - self.m11*self.m34*self.m43 -
            self.m13*self.m31*self.m44 + self.m11*self.m33*self.m44,

            self.m14*self.m23*self.m41 - self.m13*self.m24*self.m41 -
            self.m14*self.m21*self.m43 + self.m11*self.m24*self.m43 +
            self.m13*self.m21*self.m44 - self.m11*self.m23*self.m44,

            self.m13*self.m24*self.m31 - self.m14*self.m23*self.m31 +
            self.m14*self.m21*self.m33 - self.m11*self.m24*self.m33 -
            self.m13*self.m21*self.m34 + self.m11*self.m23*self.m34,

            self.m22*self.m34*self.m41 - self.m24*self.m32*self.m41 +
            self.m24*self.m31*self.m42 - self.m21*self.m34*self.m42 -
            self.m22*self.m31*self.m44 + self.m21*self.m32*self.m44,

            self.m14*self.m32*self.m41 - self.m12*self.m34*self.m41 -
            self.m14*self.m31*self.m42 + self.m11*self.m34*self.m42 +
            self.m12*self.m31*self.m44 - self.m11*self.m32*self.m44,

            self.m12*self.m24*self.m41 - self.m14*self.m22*self.m41 +
            self.m14*self.m21*self.m42 - self.m11*self.m24*self.m42 -
            self.m12*self.m21*self.m44 + self.m11*self.m22*self.m44,

            self.m14*self.m22*self.m31 - self.m12*self.m24*self.m31 -
            self.m14*self.m21*self.m32 + self.m11*self.m24*self.m32 +
            self.m12*self.m21*self.m34 - self.m11*self.m22*self.m34,

            self.m23*self.m32*self.m41 - self.m22*self.m33*self.m41 -
            self.m23*self.m31*self.m42 + self.m21*self.m33*self.m42 +
            self.m22*self.m31*self.m43 - self.m21*self.m32*self.m43,

            self.m12*self.m33*self.m41 - self.m13*self.m32*self.m41 +
            self.m13*self.m31*self.m42 - self.m11*self.m33*self.m42 -
            self.m12*self.m31*self.m43 + self.m11*self.m32*self.m43,

            self.m13*self.m22*self.m41 - self.m12*self.m23*self.m41 -
            self.m13*self.m21*self.m42 + self.m11*self.m23*self.m42 +
            self.m12*self.m21*self.m43 - self.m11*self.m22*self.m43,

            self.m12*self.m23*self.m31 - self.m13*self.m22*self.m31 +
            self.m13*self.m21*self.m32 - self.m11*self.m23*self.m32 -
            self.m12*self.m21*self.m33 + self.m11*self.m22*self.m33
        );

        let _1: T = One::one();
        Some(m.mul_s(_1 / det))
    }

    /// Compute the determinant of the matrix.
    pub fn determinant(&self) -> T {
        self.m14 * self.m23 * self.m32 * self.m41 -
        self.m13 * self.m24 * self.m32 * self.m41 -
        self.m14 * self.m22 * self.m33 * self.m41 +
        self.m12 * self.m24 * self.m33 * self.m41 +
        self.m13 * self.m22 * self.m34 * self.m41 -
        self.m12 * self.m23 * self.m34 * self.m41 -
        self.m14 * self.m23 * self.m31 * self.m42 +
        self.m13 * self.m24 * self.m31 * self.m42 +
        self.m14 * self.m21 * self.m33 * self.m42 -
        self.m11 * self.m24 * self.m33 * self.m42 -
        self.m13 * self.m21 * self.m34 * self.m42 +
        self.m11 * self.m23 * self.m34 * self.m42 +
        self.m14 * self.m22 * self.m31 * self.m43 -
        self.m12 * self.m24 * self.m31 * self.m43 -
        self.m14 * self.m21 * self.m32 * self.m43 +
        self.m11 * self.m24 * self.m32 * self.m43 +
        self.m12 * self.m21 * self.m34 * self.m43 -
        self.m11 * self.m22 * self.m34 * self.m43 -
        self.m13 * self.m22 * self.m31 * self.m44 +
        self.m12 * self.m23 * self.m31 * self.m44 +
        self.m13 * self.m21 * self.m32 * self.m44 -
        self.m11 * self.m23 * self.m32 * self.m44 -
        self.m12 * self.m21 * self.m33 * self.m44 +
        self.m11 * self.m22 * self.m33 * self.m44
    }

    /// Multiplies all of the matrix's component by a scalar and returns the result.
    pub fn mul_s(&self, x: T) -> TypedMatrix4D<T, Src, Dst> {
        TypedMatrix4D::row_major(
            self.m11 * x, self.m12 * x, self.m13 * x, self.m14 * x,
            self.m21 * x, self.m22 * x, self.m23 * x, self.m24 * x,
            self.m31 * x, self.m32 * x, self.m33 * x, self.m34 * x,
            self.m41 * x, self.m42 * x, self.m43 * x, self.m44 * x
        )
    }

    /// Convenience function to create a scale matrix from a ScaleFactor.
    pub fn from_scale_factor(scale: ScaleFactor<T, Src, Dst>) -> TypedMatrix4D<T, Src, Dst> {
        TypedMatrix4D::create_scale(scale.get(), scale.get(), scale.get())
    }

    /// Returns the given 2d point transformed by this matrix.
    ///
    /// The input point must be use the unit Src, and the returned point has the unit Dst.
    #[inline]
    pub fn transform_point(&self, p: &TypedPoint2D<T, Src>) -> TypedPoint2D<T, Dst> {
        self.transform_point4d(&TypedPoint4D::new(p.x, p.y, Zero::zero(), One::one())).to_2d()
    }

    /// Returns the given 3d point transformed by this matrix.
    ///
    /// The input point must be use the unit Src, and the returned point has the unit Dst.
    #[inline]
    pub fn transform_point3d(&self, p: &TypedPoint3D<T, Src>) -> TypedPoint3D<T, Dst> {
        self.transform_point4d(&TypedPoint4D::new(p.x, p.y, p.z, One::one())).to_3d()
    }

    /// Returns the given 4d point transformed by this matrix.
    ///
    /// The input point must be use the unit Src, and the returned point has the unit Dst.
    #[inline]
    pub fn transform_point4d(&self, p: &TypedPoint4D<T, Src>) -> TypedPoint4D<T, Dst> {
        let x = p.x * self.m11 + p.y * self.m21 + p.z * self.m31 + self.m41;
        let y = p.x * self.m12 + p.y * self.m22 + p.z * self.m32 + self.m42;
        let z = p.x * self.m13 + p.y * self.m23 + p.z * self.m33 + self.m43;
        let w = p.x * self.m14 + p.y * self.m24 + p.z * self.m34 + self.m44;
        TypedPoint4D::new(x, y, z, w)
    }

    /// Create a 3d translation matrix
    pub fn create_translation(x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        TypedMatrix4D::row_major(
            _1, _0, _0, _0,
            _0, _1, _0, _0,
            _0, _0, _1, _0,
             x,  y,  z, _1
        )
    }

    /// Returns a matrix with a translation applied before self's transformation.
    pub fn pre_translated(&self, x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        self.pre_mul(&TypedMatrix4D::create_translation(x, y, z))
    }

    /// Returns a matrix with a translation applied after self's transformation.
    pub fn post_translated(&self, x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        self.post_mul(&TypedMatrix4D::create_translation(x, y, z))
    }

    /// Create a 3d scale matrix
    pub fn create_scale(x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        TypedMatrix4D::row_major(
             x, _0, _0, _0,
            _0,  y, _0, _0,
            _0, _0,  z, _0,
            _0, _0, _0, _1
        )
    }

    /// Returns a matrix with a scale applied before self's transformation.
    pub fn pre_scaled(&self, x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        TypedMatrix4D::row_major(
            self.m11 * x, self.m12,     self.m13,     self.m14,
            self.m21    , self.m22 * y, self.m23,     self.m24,
            self.m31    , self.m32,     self.m33 * z, self.m34,
            self.m41    , self.m42,     self.m43,     self.m44
        )
    }

    /// Returns a matrix with a scale applied after self's transformation.
    pub fn post_scaled(&self, x: T, y: T, z: T) -> TypedMatrix4D<T, Src, Dst> {
        self.post_mul(&TypedMatrix4D::create_scale(x, y, z))
    }

    /// Create a 3d rotation matrix from an angle / axis.
    /// The supplied axis must be normalized.
    pub fn create_rotation(x: T, y: T, z: T, theta: Radians<T>) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        let _2 = _1 + _1;

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;

        let half_theta = theta.get() / _2;
        let sc = half_theta.sin() * half_theta.cos();
        let sq = half_theta.sin() * half_theta.sin();

        TypedMatrix4D::row_major(
            _1 - _2 * (yy + zz) * sq,
            _2 * (x * y * sq - z * sc),
            _2 * (x * z * sq + y * sc),
            _0,

            _2 * (x * y * sq + z * sc),
            _1 - _2 * (xx + zz) * sq,
            _2 * (y * z * sq - x * sc),
            _0,

            _2 * (x * z * sq - y * sc),
            _2 * (y * z * sq + x * sc),
            _1 - _2 * (xx + yy) * sq,
            _0,

            _0,
            _0,
            _0,
            _1
        )
    }

    /// Returns a matrix with a rotation applied after self's transformation.
    pub fn post_rotated(&self, x: T, y: T, z: T, theta: Radians<T>) -> TypedMatrix4D<T, Src, Dst> {
        self.post_mul(&TypedMatrix4D::create_rotation(x, y, z, theta))
    }

    /// Returns a matrix with a rotation applied before self's transformation.
    pub fn pre_rotated(&self, x: T, y: T, z: T, theta: Radians<T>) -> TypedMatrix4D<T, Src, Dst> {
        self.pre_mul(&TypedMatrix4D::create_rotation(x, y, z, theta))
    }

    /// Create a 2d skew matrix.
    ///
    /// See https://drafts.csswg.org/css-transforms/#funcdef-skew
    pub fn create_skew(alpha: Radians<T>, beta: Radians<T>) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        let (sx, sy) = (beta.get().tan(), alpha.get().tan());
        TypedMatrix4D::row_major(
            _1, sx, _0, _0,
            sy, _1, _0, _0,
            _0, _0, _1, _0,
            _0, _0, _0, _1
        )
    }

    /// Create a simple perspective projection matrix
    pub fn create_perspective(d: T) -> TypedMatrix4D<T, Src, Dst> {
        let (_0, _1): (T, T) = (Zero::zero(), One::one());
        TypedMatrix4D::row_major(
            _1, _0, _0, _0,
            _0, _1, _0, _0,
            _0, _0, _1, -_1 / d,
            _0, _0, _0, _1
        )
    }
}

impl<T: Copy, Src, Dst> TypedMatrix4D<T, Src, Dst> {
    /// Returns an array containing this matrix's terms in row-major order (the order
    /// in which the matrix is actually laid out in memory).
    pub fn to_row_major_array(&self) -> [T; 16] {
        [
            self.m11, self.m12, self.m13, self.m14,
            self.m21, self.m22, self.m23, self.m24,
            self.m31, self.m32, self.m33, self.m34,
            self.m41, self.m42, self.m43, self.m44
        ]
    }

    /// Returns an array containing this matrix's terms in column-major order.
    pub fn to_column_major_array(&self) -> [T; 16] {
        [
            self.m11, self.m21, self.m31, self.m41,
            self.m12, self.m22, self.m32, self.m42,
            self.m13, self.m23, self.m33, self.m43,
            self.m14, self.m24, self.m34, self.m44
        ]
    }

    /// Returns an array containing this matrix's 4 rows in (in row-major order)
    /// as arrays.
    ///
    /// This is a convenience method to interface with other libraries like glium.
    pub fn to_row_arrays(&self) -> [[T; 4];4] {
        [
            [self.m11, self.m12, self.m13, self.m14],
            [self.m21, self.m22, self.m23, self.m24],
            [self.m31, self.m32, self.m33, self.m34],
            [self.m41, self.m42, self.m43, self.m44]
        ]
    }

    /// Returns an array containing this matrix's 4 columns in (in row-major order,
    /// or 4 rows in column-major order) as arrays.
    ///
    /// This is a convenience method to interface with other libraries like glium.
    pub fn to_column_arrays(&self) -> [[T; 4]; 4] {
        [
            [self.m11, self.m21, self.m31, self.m41],
            [self.m12, self.m22, self.m32, self.m42],
            [self.m13, self.m23, self.m33, self.m43],
            [self.m14, self.m24, self.m34, self.m44]
        ]
    }
}

impl<T: Copy + fmt::Debug, Src, Dst> fmt::Debug for TypedMatrix4D<T, Src, Dst> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_row_major_array().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use point::Point2D;
    use Radians;
    use super::*;

    type Mf32 = Matrix4D<f32>;

    // For convenience.
    fn rad(v: f32) -> Radians<f32> { Radians::new(v) }

    #[test]
    pub fn test_ortho() {
        let (left, right, bottom, top) = (0.0f32, 1.0f32, 0.1f32, 1.0f32);
        let (near, far) = (-1.0f32, 1.0f32);
        let result = Mf32::ortho(left, right, bottom, top, near, far);
        let expected = Mf32::row_major(
             2.0,  0.0,         0.0, 0.0,
             0.0,  2.22222222,  0.0, 0.0,
             0.0,  0.0,        -1.0, 0.0,
            -1.0, -1.22222222, -0.0, 1.0
        );
        debug!("result={:?} expected={:?}", result, expected);
        assert!(result.approx_eq(&expected));
    }

    #[test]
    pub fn test_is_2d() {
        assert!(Mf32::identity().is_2d());
        assert!(Mf32::create_rotation(0.0, 0.0, 1.0, rad(0.7854)).is_2d());
        assert!(!Mf32::create_rotation(0.0, 1.0, 0.0, rad(0.7854)).is_2d());
    }

    #[test]
    pub fn test_row_major_2d() {
        let m1 = Mf32::row_major_2d(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let m2 = Mf32::row_major(
            1.0, 2.0, 0.0, 0.0,
            3.0, 4.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            5.0, 6.0, 0.0, 1.0
        );
        assert_eq!(m1, m2);
    }

    #[test]
    pub fn test_inverse_simple() {
        let m1 = Mf32::identity();
        let m2 = m1.inverse().unwrap();
        assert!(m1.approx_eq(&m2));
    }

    #[test]
    pub fn test_inverse_scale() {
        let m1 = Mf32::create_scale(1.5, 0.3, 2.1);
        let m2 = m1.inverse().unwrap();
        assert!(m1.pre_mul(&m2).approx_eq(&Mf32::identity()));
    }

    #[test]
    pub fn test_inverse_translate() {
        let m1 = Mf32::create_translation(-132.0, 0.3, 493.0);
        let m2 = m1.inverse().unwrap();
        assert!(m1.pre_mul(&m2).approx_eq(&Mf32::identity()));
    }

    #[test]
    pub fn test_inverse_rotate() {
        let m1 = Mf32::create_rotation(0.0, 1.0, 0.0, rad(1.57));
        let m2 = m1.inverse().unwrap();
        assert!(m1.pre_mul(&m2).approx_eq(&Mf32::identity()));
    }

    #[test]
    pub fn test_inverse_transform_point_2d() {
        let m1 = Mf32::create_translation(100.0, 200.0, 0.0);
        let m2 = m1.inverse().unwrap();
        assert!(m1.pre_mul(&m2).approx_eq(&Mf32::identity()));

        let p1 = Point2D::new(1000.0, 2000.0);
        let p2 = m1.transform_point(&p1);
        assert!(p2.eq(&Point2D::new(1100.0, 2200.0)));

        let p3 = m2.transform_point(&p2);
        assert!(p3.eq(&p1));
    }

    #[test]
    pub fn test_pre_post() {
        let m1 = Matrix4D::identity().post_scaled(1.0, 2.0, 3.0).post_translated(1.0, 2.0, 3.0);
        let m2 = Matrix4D::identity().pre_translated(1.0, 2.0, 3.0).pre_scaled(1.0, 2.0, 3.0);
        assert!(m1.approx_eq(&m2));
    }
}
