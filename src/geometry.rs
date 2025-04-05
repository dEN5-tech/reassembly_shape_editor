// geometry.rs - general purpose geometry and math
// - polygon intersections and computational geometry
// - vector and numerical operations

use std::f32::consts::PI;
use std::ops::{Add, Sub, Mul, Neg};

// Constants
pub const EPSILON: f32 = 0.0001;
pub const TAU: f32 = PI * 2.0;
pub const GOLDEN_RATIO: f32 = 1.61803398875;

// Type aliases for common vector types
pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type DVec2 = glam::DVec2;
pub type DVec3 = glam::DVec3;
pub type DVec4 = glam::DVec4;
pub type IVec2 = glam::IVec2;
pub type IVec3 = glam::IVec3;
pub type IVec4 = glam::IVec4;

// Short aliases
pub type F2 = Vec2;
pub type F3 = Vec3;
pub type F4 = Vec4;
pub type D2 = DVec2;
pub type D3 = DVec3;
pub type D4 = DVec4;
pub type I2 = IVec2;
pub type I3 = IVec3;
pub type I4 = IVec4;

// Ternary digit: -1, 0, or 1, (false, unknown, true)
#[derive(Debug, Clone, Copy)]
pub struct Trit {
    val: i8,
}

impl Trit {
    pub fn new() -> Self {
        Trit { val: 0 }
    }
    
    pub fn from_int(v: i32) -> Self {
        Trit { val: if v > 0 { 1 } else if v < 0 { -1 } else { 0 } }
    }
    
    pub fn from_bool(v: bool) -> Self {
        Trit { val: if v { 1 } else { -1 } }
    }
}

impl std::ops::Not for Trit {
    type Output = Trit;
    
    fn not(self) -> Self::Output {
        Trit { val: -self.val }
    }
}

impl std::ops::BitAnd for Trit {
    type Output = Trit;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        if self.val > 0 { rhs } else { self }
    }
}

impl std::ops::BitOr for Trit {
    type Output = Trit;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        if self.val > 0 { self } else { rhs }
    }
}

impl PartialEq for Trit {
    fn eq(&self, other: &Trit) -> bool {
        if self.val != 0 && other.val != 0 {
            self.val == other.val
        } else {
            false
        }
    }
}

impl PartialEq<bool> for Trit {
    fn eq(&self, other: &bool) -> bool {
        if *other {
            self.val > 0
        } else {
            self.val < 0
        }
    }
}

// Error checking functions
#[inline]
pub fn fpu_error(x: f32) -> bool {
    x.is_nan() || x.is_infinite()
}

#[inline]
pub fn fpu_error_vec2(v: Vec2) -> bool {
    fpu_error(v.x) || fpu_error(v.y)
}

#[inline]
pub fn fpu_error_vec3(v: Vec3) -> bool {
    fpu_error(v.x) || fpu_error(v.y) || fpu_error(v.z)
}

// Custom min and max functions for Vec2 since it doesn't implement Ord
#[inline]
pub fn vec2_min(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x.min(b.x), a.y.min(b.y))
}

#[inline]
pub fn vec2_max(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x.max(b.x), a.y.max(b.y))
}

// Rounding functions

/// Round float to nearest multiple of v
#[inline]
pub fn round_to(a: f32, v: f32) -> f32 {
    if v.abs() < EPSILON {
        return a;
    }
    v * (a / v).round()
}

/// Round Vec2 to nearest multiple of v
#[inline]
pub fn round_vec2(a: Vec2, v: f32) -> Vec2 {
    Vec2::new(round_to(a.x, v), round_to(a.y, v))
}

/// Round double to nearest multiple of v
#[inline]
pub fn round_to_f64(a: f64, v: f64) -> f64 {
    if v.abs() < EPSILON as f64 {
        return a;
    }
    v * (a / v).round()
}

/// Round DVec2 to nearest multiple of v
#[inline]
pub fn round_dvec2(a: DVec2, v: f64) -> DVec2 {
    DVec2::new(round_to_f64(a.x, v), round_to_f64(a.y, v))
}

/// Round up integer to nearest multiple
#[inline]
pub fn round_up(num: i32, mult: i32) -> i32 {
    ((num + mult - 1) / mult) * mult
}

/// Round down integer to nearest multiple
#[inline]
pub fn round_down(num: i32, mult: i32) -> i32 {
    (num / mult) * mult
}

/// Round up to the next power of 2
#[inline]
pub fn round_up_power2(v: u32) -> u32 {
    let mut i = 1;
    while i < v {
        i *= 2;
    }
    i
}

/// Ceil float to nearest multiple of v
#[inline]
pub fn ceil_to(a: f32, v: f32) -> f32 {
    if v.abs() < EPSILON {
        return a;
    }
    v * (a / v).ceil()
}

/// Floor float to nearest multiple of v
#[inline]
pub fn floor_to(a: f32, v: f32) -> f32 {
    if v.abs() < EPSILON {
        return a;
    }
    v * (a / v).floor()
}

/// Ceil Vec2 to nearest multiple of v
#[inline]
pub fn ceil_vec2(a: Vec2, v: f32) -> Vec2 {
    Vec2::new(ceil_to(a.x, v), ceil_to(a.y, v))
}

/// Floor Vec2 to nearest multiple of v
#[inline]
pub fn floor_vec2(a: Vec2, v: f32) -> Vec2 {
    Vec2::new(floor_to(a.x, v), floor_to(a.y, v))
}

/// Convert float to int using floor
#[inline]
pub fn floor_int(f: f32) -> i32 {
    debug_assert!(f.abs() < (2 << 23) as f32);
    let i = f as i32;
    if f < 0.0 && f != i as f32 { i - 1 } else { i }
}

/// Convert float to int using ceil
#[inline]
pub fn ceil_int(f: f32) -> i32 {
    debug_assert!(f.abs() < (2 << 23) as f32);
    let i = f as i32;
    if f >= 0.0 && f != i as f32 { i + 1 } else { i }
}

/// Convert float to int using round
#[inline]
pub fn round_int(f: f32) -> i32 {
    debug_assert!(f.abs() < (2 << 23) as f32);
    if f >= 0.0 {
        (f + 0.49999997).floor() as i32
    } else {
        (f - 0.50000003).ceil() as i32
    }
}

/// Convert Vec2 to IVec2 using floor
#[inline]
pub fn floor_ivec2(f: Vec2) -> IVec2 {
    IVec2::new(floor_int(f.x), floor_int(f.y))
}

/// Convert Vec2 to IVec2 using ceil
#[inline]
pub fn ceil_ivec2(f: Vec2) -> IVec2 {
    IVec2::new(ceil_int(f.x), ceil_int(f.y))
}

/// Convert Vec2 to IVec2 using round
#[inline]
pub fn round_ivec2(f: Vec2) -> IVec2 {
    IVec2::new(round_int(f.x), round_int(f.y))
}

// Angle and Vector Functions

/// Convert angle to unit vector
#[inline]
pub fn angle_to_vector(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

/// Convert vector to angle
#[inline]
pub fn vector_to_angle(vec: Vec2) -> f32 {
    vec.y.atan2(vec.x)
}

/// Convert angle to unit vector (f64 version)
#[inline]
pub fn angle_to_vector_f64(angle: f64) -> DVec2 {
    DVec2::new(angle.cos(), angle.sin())
}

/// Convert vector to angle (f64 version)
#[inline]
pub fn vector_to_angle_f64(vec: DVec2) -> f64 {
    vec.y.atan2(vec.x)
}

/// Short alias for angle_to_vector
#[inline]
pub fn a2v(angle: f32) -> Vec2 {
    angle_to_vector(angle)
}

/// Short alias for vector_to_angle
#[inline]
pub fn v2a(vec: Vec2) -> f32 {
    vector_to_angle(vec)
}

/// Short alias for angle_to_vector (f64 version)
#[inline]
pub fn a2v_f64(angle: f64) -> DVec2 {
    angle_to_vector_f64(angle)
}

/// Short alias for vector_to_angle (f64 version)
#[inline]
pub fn v2a_f64(vec: DVec2) -> f64 {
    vector_to_angle_f64(vec)
}

/// Return [-1, 1] indicating how closely the angles are aligned
#[inline]
pub fn dot_angles(a: f32, b: f32) -> f32 {
    (a - b).cos()
}

/// Return squared value
#[inline]
pub fn squared<T>(val: T) -> T 
where T: Mul<Output = T> + Copy
{
    val * val
}

/// Return sign of value as -1, 0, or 1
#[inline]
pub fn sign<T>(val: T) -> T 
where T: PartialOrd + Sub<Output = T> + From<i8> + Copy
{
    let zero = T::from(0);
    if val > zero {
        T::from(1)
    } else if val < zero {
        T::from(-1)
    } else {
        zero
    }
}

/// Return sign of Vec2 components
#[inline]
pub fn sign_vec2(v: Vec2) -> Vec2 {
    Vec2::new(sign(v.x), sign(v.y))
}

/// Return sign of Vec3 components
#[inline]
pub fn sign_vec3(v: Vec3) -> Vec3 {
    Vec3::new(sign(v.x), sign(v.y), sign(v.z))
}

/// Return sign of Vec4 components
#[inline]
pub fn sign_vec4(v: Vec4) -> Vec4 {
    Vec4::new(sign(v.x), sign(v.y), sign(v.z), sign(v.w))
}

/// Return sign of value with threshold
#[inline]
pub fn sign_int<T>(val: T, threshold: T) -> i8
where T: PartialOrd + Neg<Output = T> + Copy
{
    if val > threshold {
        1
    } else if val < -threshold {
        -1
    } else {
        0
    }
}

/// Rotate vector 90 degrees counter-clockwise
#[inline]
pub fn rotate90(v: Vec2) -> Vec2 {
    Vec2::new(-v.y, v.x)
}

/// Rotate vector 90 degrees clockwise
#[inline]
pub fn rotate_n90(v: Vec2) -> Vec2 {
    Vec2::new(v.y, -v.x)
}

/// Calculate distance between two integers
#[inline]
pub fn distance_int(a: i32, b: i32) -> i32 {
    (a - b).abs()
}

/// Create Vec2 with golden ratio (y to x)
#[inline]
pub fn to_golden_ratio_y(y: f32) -> Vec2 {
    Vec2::new(y * GOLDEN_RATIO, y)
}

/// Create Vec2 with golden ratio (x to y)
#[inline]
pub fn to_golden_ratio_x(x: f32) -> Vec2 {
    Vec2::new(x, x / GOLDEN_RATIO)
}

/// Calculate 2D cross product
#[inline]
pub fn cross_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

/// Clamp a value between min and max
#[inline]
pub fn clamp<T>(v: T, mn: T, mx: T) -> T
where T: PartialOrd
{
    if v < mn { mn } else if v > mx { mx } else { v }
}

/// Clamp a Vec2 between min and max vectors
#[inline]
pub fn clamp_vec2(v: Vec2, mn: Vec2, mx: Vec2) -> Vec2 {
    Vec2::new(
        clamp(v.x, mn.x, mx.x),
        clamp(v.y, mn.y, mx.y)
    )
}

/// Clamp the length of a vector between min and max
#[inline]
pub fn clamp_length(v: Vec2, mn: f32, mx: f32) -> Vec2 {
    let len = v.length();
    if len < mn {
        v * (mn / len)
    } else if len > mx {
        v * (mx / len)
    } else {
        v
    }
}

/// Clamp the magnitude of a floating point value between min and max
#[inline]
pub fn clamp_mag(v: f32, mn: f32, mx: f32) -> f32 {
    let vm = v.abs();
    if vm < mn {
        if v > 0.0 { mn } else { -mn }
    } else if vm > mx {
        if v > 0.0 { mx } else { -mx }
    } else {
        v
    }
}

/// Get maximum dimension of Vec2
#[inline]
pub fn max_dim(v: Vec2) -> f32 {
    v.x.max(v.y)
}

/// Get minimum dimension of Vec2
#[inline]
pub fn min_dim(v: Vec2) -> f32 {
    v.x.min(v.y)
}

/// Check if Vec2 is near zero
#[inline]
pub fn near_zero(v: Vec2) -> bool {
    v.x.abs() <= EPSILON && v.y.abs() <= EPSILON
}

/// Check if Vec3 is near zero
#[inline]
pub fn near_zero_vec3(v: Vec3) -> bool {
    v.x.abs() <= EPSILON && v.y.abs() <= EPSILON && v.z.abs() <= EPSILON
}

/// Check if float is near zero
#[inline]
pub fn near_zero_f32(v: f32) -> bool {
    v.abs() <= EPSILON
}

/// Modulo for integers that works with negative numbers
#[inline]
pub fn modulo(x: i32, y: i32) -> i32 {
    debug_assert!(y > 0);
    
    if x >= 0 {
        return x % y;
    }
    
    let m = x - y * (x / y);
    if m < 0 {
        y + m
    } else if m == y {
        0
    } else {
        m
    }
}

/// Modulo for floats that works with negative numbers
#[inline]
pub fn modulo_f32(x: f32, y: f32) -> f32 {
    let m = x - y * (x / y).floor();
    
    if y > 0.0 {
        if m >= y {
            0.0
        } else if m < 0.0 {
            if y + m == y { 0.0 } else { y + m }
        } else {
            m
        }
    } else {
        if m <= y {
            0.0
        } else if m > 0.0 {
            if y + m == y { 0.0 } else { y + m }
        } else {
            m
        }
    }
}

/// Modulo for Vec2
#[inline]
pub fn modulo_vec2(val: Vec2, div: Vec2) -> Vec2 {
    Vec2::new(modulo_f32(val.x, div.x), modulo_f32(val.y, div.y))
}

/// Modulo for Vec2 with scalar divisor
#[inline]
pub fn modulo_vec2_scalar(val: Vec2, div: f32) -> Vec2 {
    Vec2::new(modulo_f32(val.x, div), modulo_f32(val.y, div))
}

/// Return value with smallest absolute value
#[inline]
pub fn min_abs(a: f32, b: f32) -> f32 {
    if a.abs() <= b.abs() { a } else { b }
}

/// Return value with largest absolute value
#[inline]
pub fn max_abs(a: f32, b: f32) -> f32 {
    if a.abs() >= b.abs() { a } else { b }
}

/// Return Vec2 with min absolute values
#[inline]
pub fn min_abs_vec2(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(min_abs(a.x, b.x), min_abs(a.y, b.y))
}

/// Return Vec2 with max absolute values
#[inline]
pub fn max_abs_vec2(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(max_abs(a.x, b.x), max_abs(a.y, b.y))
}

/// Return shortest signed difference between angles [0, pi]
#[inline]
pub fn distance_angles(a: f32, b: f32) -> f32 {
    modulo_f32(b - a + 1.5 * TAU, TAU) - PI
}

/// Normalize vector safely (prevents NaN)
#[inline]
pub fn normalize_safe(a: Vec2) -> Vec2 {
    if near_zero(a) {
        debug_assert!(false, "length < epsilon: [{}, {}]", a.x, a.y);
        a
    } else {
        a.normalize()
    }
}

/// Normalize vector safely (returns zero if near-zero input)
#[inline]
pub fn normalize_or_zero(a: Vec2) -> Vec2 {
    if near_zero(a) {
        Vec2::ZERO
    } else {
        a.normalize()
    }
}

/// Raises each component of a Vec2 to power e
#[inline]
pub fn pow_vec2(v: Vec2, e: f32) -> Vec2 {
    Vec2::new(v.x.powf(e), v.y.powf(e))
}

/// Raises each component of a Vec3 to power e
#[inline]
pub fn pow_vec3(v: Vec3, e: f32) -> Vec3 {
    Vec3::new(v.x.powf(e), v.y.powf(e), v.z.powf(e))
}

/// Limit vector length to maximum
#[inline]
pub fn max_length(v: Vec2, max: f32) -> Vec2 {
    let l = v.length();
    if l > max {
        v * (max / l)
    } else {
        v
    }
}

/// Enforce minimum vector length
#[inline]
pub fn min_length(v: Vec2, min: f32) -> Vec2 {
    let l = v.length();
    if l < min {
        v * (min / l)
    } else {
        v
    }
}

/// Calculate squared length of Vec2
#[inline]
pub fn length_sqr(a: Vec2) -> f32 {
    a.x * a.x + a.y * a.y
}

/// Calculate squared distance between two Vec2
#[inline]
pub fn distance_sqr(a: Vec2, b: Vec2) -> f32 {
    length_sqr(a - b)
}

/// Calculate squared length of Vec3
#[inline]
pub fn length_sqr_vec3(a: Vec3) -> f32 {
    a.x * a.x + a.y * a.y + a.z * a.z
}

/// Calculate squared distance between two Vec3
#[inline]
pub fn distance_sqr_vec3(a: Vec3, b: Vec3) -> f32 {
    length_sqr_vec3(a - b)
}

/// Convert radians to degrees
#[inline]
pub fn to_degrees(radians: f32) -> f32 {
    radians * 180.0 / PI
}

/// Convert degrees to radians
#[inline]
pub fn to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

/// Rotate vector v by angle a
#[inline]
pub fn rotate(v: Vec2, a: f32) -> Vec2 {
    let cosa = a.cos();
    let sina = a.sin();
    Vec2::new(
        cosa * v.x - sina * v.y,
        sina * v.x + cosa * v.y
    )
}

/// Rotate vector v counter-clockwise by vector a
#[inline]
pub fn rotate_vec(v: Vec2, a: Vec2) -> Vec2 {
    Vec2::new(
        a.x * v.x - a.y * v.y,
        a.y * v.x + a.x * v.y
    )
}

/// Rotate vector v clockwise by vector a
#[inline]
pub fn rotate_vec_clockwise(v: Vec2, a: Vec2) -> Vec2 {
    Vec2::new(
        a.x * v.x + a.y * v.y,
        -a.y * v.x + a.x * v.y
    )
}

/// Swap X and Y components
#[inline]
pub fn swap_xy(v: Vec2) -> Vec2 {
    Vec2::new(v.y, v.x)
}

/// Flip Y component
#[inline]
pub fn flip_y(v: f32) -> Vec2 {
    Vec2::new(v, -v)
}

/// Flip X component
#[inline]
pub fn flip_x(v: f32) -> Vec2 {
    Vec2::new(-v, v)
}

/// Flip Y component of Vec2
#[inline]
pub fn flip_y_vec2(v: Vec2) -> Vec2 {
    Vec2::new(v.x, -v.y)
}

/// Flip X component of Vec2
#[inline]
pub fn flip_x_vec2(v: Vec2) -> Vec2 {
    Vec2::new(-v.x, v.y)
}

/// Create Vec2 with only Y component
#[inline]
pub fn just_y(v: impl Into<f32>) -> Vec2 {
    Vec2::new(0.0, v.into())
}

/// Create Vec2 with only X component
#[inline]
pub fn just_x(v: impl Into<f32>) -> Vec2 {
    Vec2::new(v.into(), 0.0)
}

/// Create Vec3 with only Z component
#[inline]
pub fn just_z(v: impl Into<f32>) -> Vec3 {
    Vec3::new(0.0, 0.0, v.into())
}

/// Linear interpolation between from and to values
#[inline]
pub fn lerp<T>(from: T, to: T, v: f32) -> T
where T: Add<Output = T> + Mul<f32, Output = T> + Copy
{
    from * (1.0 - v) + to * v
}

/// Clamped linear interpolation (ensures v is between 0 and 1)
#[inline]
pub fn clamp_lerp<T>(from: T, to: T, v: f32) -> T
where T: Add<Output = T> + Mul<f32, Output = T> + Copy
{
    lerp(from, to, clamp(v, 0.0, 1.0))
}

/// Linear interpolation between angles
#[inline]
pub fn lerp_angles(a: f32, b: f32, v: f32) -> f32 {
    vector_to_angle(lerp(angle_to_vector(a), angle_to_vector(b), v))
}

/// Inverse linear interpolation - returns ratio between a and b where val falls
#[inline]
pub fn inv_lerp<T>(zero: T, one: T, val: T) -> f32
where T: Sub<Output = T> + Copy, f32: From<T>
{
    let denom = f32::from(one - zero);
    if near_zero_f32(denom) {
        return 0.0;
    }
    f32::from(val - zero) / denom
}

/// Normalized sigmoid (s-shape) function
#[inline]
pub fn signorm(x: f32, k: f32) -> f32 {
    let mut y = 0.0;
    let mut x = x;
    let mut k = k;
    
    if x > 0.5 {
        x -= 0.5;
        k = -1.0 - k;
        y = 0.5;
    }
    
    y + (2.0 * x * k) / (2.0 * (1.0 + k - 2.0 * x))
}

/// Smooth step implementation
#[inline]
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// Map unorm to bell curve (0->0, 0.5->1, 1->0)
#[inline]
pub fn bellcurve(x: f32) -> f32 {
    0.5 * (-f32::cos(TAU * x) + 1.0)
}

/// Gaussian distribution
#[inline]
pub fn gaussian(x: f32, stdev: f32) -> f32 {
    let sqrt_2pi = 2.5066282746310002;
    f32::exp(-(x * x) / (2.0 * stdev * stdev)) / (stdev * sqrt_2pi)
}

/// Check if point is within range
#[inline]
pub fn is_in_range<T>(p: T, mn: T, mx: T) -> bool
where T: PartialOrd
{
    mn <= p && p < mx
}

/// Check if Vec2 is within range
#[inline]
pub fn is_in_range_vec2(p: Vec2, mn: Vec2, mx: Vec2) -> bool {
    is_in_range(p.x, mn.x, mx.x) && is_in_range(p.y, mn.y, mx.y)
}

/// Check if point is inside circle
#[inline]
pub fn intersect_point_circle(p: Vec2, c: Vec2, r: f32) -> bool {
    let x = p - c;
    x.x * x.x + x.y * x.y <= r * r
}

/// Check if point is inside ring
#[inline]
pub fn intersect_point_ring(p: Vec2, c: Vec2, min_r: f32, max_r: f32) -> bool {
    let x = p - c;
    let v = x.x * x.x + x.y * x.y;
    (min_r * min_r) <= v && v <= (max_r * max_r)
}

/// Check if two circles intersect
#[inline]
pub fn intersect_circle_circle(p: Vec2, pr: f32, c: Vec2, cr: f32) -> bool {
    intersect_point_circle(p, c, pr + cr)
}

/// Intersect two circles
pub fn intersect_circle_circle_points(p: Vec2, pr: f32, c: Vec2, cr: f32) -> Vec<Vec2> {
    let mut results = Vec::new();
    
    let d = p.distance(c);
    
    // Circles are far apart, no intersection
    if d > pr + cr {
        return results;
    }
    
    // One circle is inside the other
    if d < (pr - cr).abs() {
        return results;
    }
    
    // Circles are coincident
    if d < EPSILON && (pr - cr).abs() < EPSILON {
        // Infinite solutions, but we return none
        return results;
    }
    
    // Calculate intersection points
    let a = (pr * pr - cr * cr + d * d) / (2.0 * d);
    let h = f32::sqrt(pr * pr - a * a);
    
    let p2 = p + (c - p) * (a / d);
    
    // Two intersection points
    let p3a = Vec2::new(
        p2.x + h * (c.y - p.y) / d,
        p2.y - h * (c.x - p.x) / d
    );
    
    let p3b = Vec2::new(
        p2.x - h * (c.y - p.y) / d,
        p2.y + h * (c.x - p.x) / d
    );
    
    results.push(p3a);
    results.push(p3b);
    
    results
}

/// Intersect line segments a1-a2 and b1-b2
pub fn intersect_segment_segment(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let d1 = cross_2d(b2 - b1, a1 - b1);
    let d2 = cross_2d(b2 - b1, a2 - b1);
    
    // First segment doesn't intersect the line of the second segment
    if (d1 > 0.0 && d2 > 0.0) || (d1 < 0.0 && d2 < 0.0) {
        return false;
    }
    
    let d3 = cross_2d(a2 - a1, b1 - a1);
    let d4 = cross_2d(a2 - a1, b2 - a1);
    
    // Second segment doesn't intersect the line of the first segment
    if (d3 > 0.0 && d4 > 0.0) || (d3 < 0.0 && d4 < 0.0) {
        return false;
    }
    
    // Special case where the segments are collinear
    if near_zero_f32(d1) && near_zero_f32(d2) && near_zero_f32(d3) && near_zero_f32(d4) {
        // Check if the segments overlap
        let t0 = (b1 - a1).dot(a2 - a1) / length_sqr(a2 - a1);
        let t1 = t0 + (b2 - b1).dot(a2 - a1) / length_sqr(a2 - a1);
        
        let overlap = (t0 >= 0.0 && t0 <= 1.0) || 
                      (t1 >= 0.0 && t1 <= 1.0) ||
                      (t0 <= 0.0 && t1 >= 1.0) ||
                      (t1 <= 0.0 && t0 >= 1.0);
                      
        return overlap;
    }
    
    return true;
}

/// Intersect line segments a1-a2 and b1-b2, returning intersection point
pub fn intersect_segment_segment_point(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    if !intersect_segment_segment(a1, a2, b1, b2) {
        return None;
    }
    
    // Calculate intersection point
    let denom = cross_2d(a2 - a1, b2 - b1);
    
    // Handle parallel lines
    if near_zero_f32(denom) {
        // Return midpoint of overlap for collinear segments
        if near_zero_f32(cross_2d(a2 - a1, b1 - a1)) && near_zero_f32(cross_2d(a2 - a1, b2 - a1)) {
            // Find overlap
            let aa_len = length_sqr(a2 - a1);
            let t0 = (b1 - a1).dot(a2 - a1) / aa_len;
            let t1 = (b2 - a1).dot(a2 - a1) / aa_len;
            
            let t_min = t0.min(t1).max(0.0).min(1.0);
            let t_max = t0.max(t1).max(0.0).min(1.0);
            
            if t_min <= t_max {
                let t_mid = (t_min + t_max) / 2.0;
                return Some(a1 + (a2 - a1) * t_mid);
            }
        }
        return None;
    }
    
    let ua = cross_2d(b2 - b1, a1 - b1) / denom;
    
    return Some(a1 + (a2 - a1) * ua);
}

/// Check if a point is inside a polygon defined by points
pub fn intersect_poly_point(points: &[Vec2], point: Vec2) -> bool {
    if points.len() < 3 {
        return false;
    }
    
    // Ray casting algorithm
    let mut inside = false;
    let mut j = points.len() - 1;
    
    for i in 0..points.len() {
        if ((points[i].y > point.y) != (points[j].y > point.y)) &&
           (point.x < points[i].x + (points[j].x - points[i].x) * 
            (point.y - points[i].y) / (points[j].y - points[i].y)) {
            inside = !inside;
        }
        j = i;
    }
    
    inside
}

/// Check if a circle intersects with a polygon
pub fn intersect_poly_circle(points: &[Vec2], center: Vec2, radius: f32) -> bool {
    if points.len() < 3 {
        return false;
    }
    
    // Check if center is inside the polygon
    if intersect_poly_point(points, center) {
        return true;
    }
    
    // Check if any edge intersects the circle
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        if intersect_segment_circle(points[i], points[j], center, radius) {
            return true;
        }
    }
    
    false
}

/// Return true if line segment intersects circle
pub fn intersect_segment_circle(a: Vec2, b: Vec2, c: Vec2, r: f32) -> bool {
    let closest = closest_point_on_segment(a, b, c);
    intersect_point_circle(closest, c, r)
}

/// Return the point on line segment a-b closest to point p
pub fn closest_point_on_segment(a: Vec2, b: Vec2, p: Vec2) -> Vec2 {
    let segment = b - a;
    let seg_len = segment.length();
    
    if seg_len < EPSILON {
        return a;
    }
    
    let unit_seg = segment / seg_len;
    let proj = (p - a).dot(unit_seg);
    
    if proj <= 0.0 {
        return a;
    } else if proj >= seg_len {
        return b;
    } else {
        return a + unit_seg * proj;
    }
}

/// Return the point on ray starting at point a in direction dir closest to point p
pub fn closest_point_on_ray(a: Vec2, dir: Vec2, p: Vec2) -> Vec2 {
    if near_zero(dir) {
        return a;
    }
    
    let unit_dir = normalize_or_zero(dir);
    let proj = (p - a).dot(unit_dir);
    
    if proj <= 0.0 {
        return a;
    } else {
        return a + unit_dir * proj;
    }
}

/// Return true if ray starting at point E in direction d intersects circle at point C with radius r
pub fn intersect_ray_circle(e: Vec2, d: Vec2, c: Vec2, r: f32) -> bool {
    let closest = closest_point_on_ray(e, d, c);
    intersect_point_circle(closest, c, r)
}

/// Intersect ray with circle, returning intersection points
pub fn intersect_ray_circle_points(e: Vec2, d: Vec2, c: Vec2, r: f32) -> Vec<Vec2> {
    let mut results = Vec::new();
    
    // Normalize direction
    let n = normalize_or_zero(d);
    if near_zero(n) {
        return results;
    }
    
    // Vector from ray origin to circle center
    let ec = c - e;
    
    // Distance along ray to closest point to circle center
    let t_closest = ec.dot(n);
    
    // Closest point on ray to circle center
    let closest = e + n * t_closest;
    
    // Distance from closest point to circle center
    let dist = (closest - c).length();
    
    // Ray doesn't intersect circle
    if dist > r {
        return results;
    }
    
    // Distance from closest point to intersection point(s)
    let dt = f32::sqrt(r * r - dist * dist);
    
    // First intersection point (always exists if closest point is outside the circle)
    if t_closest - dt >= 0.0 {
        results.push(e + n * (t_closest - dt));
    }
    
    // Second intersection point
    if t_closest + dt >= 0.0 && dt > EPSILON {
        results.push(e + n * (t_closest + dt));
    }
    
    results
}

/// Check if ray intersects line segment
pub fn intersect_ray_segment(ray_pt: Vec2, ray_dir: Vec2, sa: Vec2, sb: Vec2) -> bool {
    let ray_normal = rotate90(ray_dir);
    let side_a = (sa - ray_pt).dot(ray_normal);
    let side_b = (sb - ray_pt).dot(ray_normal);
    
    // Segment is on one side of ray - no intersection
    if (side_a > 0.0 && side_b > 0.0) || (side_a < 0.0 && side_b < 0.0) {
        return false;
    }
    
    // Check if intersection is in positive ray direction
    let t = cross_2d(sa - ray_pt, sb - sa) / cross_2d(ray_dir, sb - sa);
    return t >= 0.0;
}

/// Check if two rectangles intersect
#[inline]
pub fn intersect_rectangle_rectangle(a: Vec2, ar: Vec2, b: Vec2, br: Vec2) -> bool {
    let delta = (a - b).abs();
    delta.x <= (ar.x + br.x) && delta.y <= (ar.y + br.y)
}

/// Check if circle intersects rectangle
#[inline]
pub fn intersect_circle_rectangle(circle: Vec2, circle_r: f32, rect_pos: Vec2, rect_half_size: Vec2) -> bool {
    let circle_distance = (circle - rect_pos).abs();
    
    // Circle is too far away from rectangle
    if circle_distance.x > (rect_half_size.x + circle_r) || 
       circle_distance.y > (rect_half_size.y + circle_r) {
        return false;
    }
    
    // Circle center is within extended rectangle
    if circle_distance.x <= rect_half_size.x || 
       circle_distance.y <= rect_half_size.y {
        return true;
    }
    
    // Check if circle intersects rectangle corner
    let corner_dist_sq = (circle_distance - rect_half_size).length_squared();
    return corner_dist_sq <= (circle_r * circle_r);
}

/// Check if point is inside rectangle
#[inline]
pub fn intersect_point_rectangle(p: Vec2, b: Vec2, br: Vec2) -> bool {
    let v = b - p;
    v.x > -br.x && v.y > -br.y && v.x <= br.x && v.y <= br.y
}

/// Check if point is inside rectangle defined by corners
#[inline]
pub fn intersect_point_rectangle_corners(p: Vec2, a: Vec2, b: Vec2) -> bool {
    let mn = vec2_min(a, b);
    let mx = vec2_max(a, b);
    p.x >= mn.x && p.y >= mn.y && p.x <= mx.x && p.y <= mx.y
}

/// Check if circle is fully contained inside rectangle
#[inline]
pub fn contained_circle_in_rectangle(circle: Vec2, circle_r: f32, rect_pos: Vec2, rect_half_size: Vec2) -> bool {
    intersect_point_rectangle(circle, rect_pos, rect_half_size - Vec2::splat(circle_r))
}

/// Axis-aligned bounding box
#[derive(Debug, Clone)]
pub struct AABBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        AABBox { min, max }
    }
    
    pub fn largest() -> Self {
        AABBox { 
            min: Vec2::new(-f32::MAX, -f32::MAX), 
            max: Vec2::new(f32::MAX, f32::MAX) 
        }
    }
    
    pub fn get_radius(&self) -> Vec2 {
        0.5 * (self.max - self.min)
    }
    
    pub fn get_center(&self) -> Vec2 {
        0.5 * (self.max + self.min)
    }
    
    pub fn get_b_radius(&self) -> f32 {
        self.get_radius().length()
    }
    
    pub fn empty(&self) -> bool {
        self.min == self.max
    }
    
    pub fn get_area(&self) -> f32 {
        let rad = self.get_radius();
        4.0 * rad.x * rad.y
    }
    
    pub fn rotated(&self, angle: f32) -> Self {
        let mut bb = AABBox::new(Vec2::ZERO, Vec2::ZERO);
        let rot = angle_to_vector(angle);
        
        bb.insert_point(rotate_vec(self.max, rot));
        bb.insert_point(rotate_vec(self.min, rot));
        bb.insert_point(rotate_vec(Vec2::new(self.max.x, self.min.y), rot));
        bb.insert_point(rotate_vec(Vec2::new(self.min.x, self.max.y), rot));
        
        bb
    }
    
    pub fn translated(&self, vec: Vec2) -> Self {
        AABBox::new(self.min + vec, self.max + vec)
    }
    
    pub fn merged(&self, other: &AABBox) -> Self {
        AABBox::new(
            vec2_min(self.min, other.min),
            vec2_max(self.max, other.max)
        )
    }
    
    pub fn start(&mut self, pt: Vec2) {
        if self.min != Vec2::ZERO || self.max != Vec2::ZERO {
            return;
        }
        self.min = pt;
        self.max = pt;
    }
    
    pub fn reset(&mut self) {
        self.min = Vec2::ZERO;
        self.max = Vec2::ZERO;
    }
    
    pub fn insert_point(&mut self, pt: Vec2) {
        self.start(pt);
        self.max = vec2_max(self.max, pt);
        self.min = vec2_min(self.min, pt);
    }
    
    pub fn insert_circle(&mut self, pt: Vec2, rad: f32) {
        self.start(pt);
        self.max = vec2_max(self.max, pt + Vec2::splat(rad));
        self.min = vec2_min(self.min, pt - Vec2::splat(rad));
    }
    
    pub fn insert_rect(&mut self, pt: Vec2, rad: Vec2) {
        self.start(pt);
        self.max = vec2_max(self.max, pt + rad);
        self.min = vec2_min(self.min, pt - rad);
    }
    
    pub fn insert_rect_corners(&mut self, p0: Vec2, p1: Vec2) {
        self.start(p0);
        self.max = vec2_max(self.max, vec2_max(p0, p1));
        self.min = vec2_min(self.min, vec2_min(p0, p1));
    }
    
    pub fn insert_poly(&mut self, pts: &[Vec2]) {
        if pts.is_empty() {
            return;
        }
        
        self.start(pts[0]);
        for pt in pts.iter().skip(1) {
            self.max = vec2_max(self.max, *pt);
            self.min = vec2_min(self.min, *pt);
        }
    }
    
    pub fn insert_aabbox(&mut self, bb: &AABBox) {
        self.start(bb.get_center());
        self.max = vec2_max(self.max, bb.max);
        self.min = vec2_min(self.min, bb.min);
    }
    
    pub fn intersect_point(&self, pt: Vec2) -> bool {
        intersect_point_rectangle(pt, self.get_center(), self.get_radius())
    }
    
    pub fn intersect_circle(&self, pt: Vec2, r: f32) -> bool {
        intersect_circle_rectangle(pt, r, self.get_center(), self.get_radius())
    }
}

/// Return orientation of three points: positive for CCW, negative for CW, zero for collinear
#[inline]
pub fn orient(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
}

/// Return orientation with p1 at origin
#[inline]
pub fn orient2(p2: Vec2, p3: Vec2) -> f32 {
    p2.x * p3.y - p2.y * p3.x
}

/// Compute the area of a polygon
#[inline]
pub fn area_for_poly(verts: &[Vec2]) -> f32 {
    if verts.len() < 3 {
        return 0.0;
    }
    
    let mut area = 0.0;
    for i in 0..verts.len() {
        let j = (i + 1) % verts.len();
        area += cross_2d(verts[i], verts[j]);
    }
    
    -area / 2.0
}

/// Compute the moment of inertia of a polygon
pub fn moment_for_poly(mass: f32, verts: &[Vec2], offset: Vec2) -> f32 {
    if verts.len() < 3 {
        return 0.0;
    }
    
    let mut sum1 = 0.0;
    let mut sum2 = 0.0;
    
    for i in 0..verts.len() {
        let v1 = verts[i] - offset;
        let v2 = verts[(i + 1) % verts.len()] - offset;
        
        let a = cross_2d(v2, v1);
        let b = v1.dot(v1) + v1.dot(v2) + v2.dot(v2);
        
        sum1 += a * b;
        sum2 += a;
    }
    
    return mass * sum1 / (6.0 * sum2);
}

/// Regular polygon apothem (inradius) given circumradius
#[inline]
pub fn regpoly_apothem(n: i32, r: f32) -> f32 {
    r * f32::cos(PI / n as f32)
}

/// Regular polygon circumradius given apothem (inradius)
#[inline]
pub fn regpoly_circumradius(n: i32, r: f32) -> f32 {
    r / f32::cos(PI / n as f32)
}

/// Regular polygon radius from side length
#[inline]
pub fn regpoly_radius_from_side(n: i32, s: f32) -> f32 {
    s / (2.0 * f32::sin(PI / n as f32))
}

/// Regular polygon area
#[inline]
pub fn regpoly_area(n: i32, r: f32, r1: f32) -> f32 {
    let r1 = if r1 == 0.0 { r } else { r1 };
    0.5 * n as f32 * r * r1 * f32::sin(TAU / n as f32)
}

/// Regular polygon perimeter
#[inline]
pub fn regpoly_perimeter(n: i32, r: f32) -> f32 {
    n as f32 * 2.0 * r * f32::sin(PI / n as f32)
} 