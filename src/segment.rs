use vector2d::Vector2D;

use roots::find_roots_cubic;
use roots::find_roots_quadratic;
//use roots::Roots;

#[derive(Debug)]
pub enum SegmentType {
    Origin,
    Line,
    Quad,
    Cubic,
    Close
}

#[derive(Debug)]
pub struct Segment {
    pub segment_type: SegmentType,
    // All segments have at least one point. In the case of a segment with
    // less than three points, the remaining values do not matter

    // Note that all points implicitly take the last point of the previous
    // segment as their starting point.

    // Start point:
    // Everything except 'Close'
    pub x_start: f32,
    pub y_start: f32,

    // Internal point 1:
    // Quads and Cubics
    pub x1: f32,
    pub y1: f32,

    // Internal point 2:
    // Only Cubics
    pub x2: f32,
    pub y2: f32,

    // End point:
    // Everything except 'Close'
    pub x_end: f32,
    pub y_end: f32,
}

impl Segment {
    pub fn intersect(&self, x: f32, y: f32, dx: f32, dy: f32) -> i32 {
        // Returns the number of times a ray with origin (x, y) and direction (dx, dy) intersections the segment.

        // This violates OOP patterns, each type should implement its own intersect method, but it works for now.
        match self.segment_type {
            SegmentType::Origin { .. } => {
                return 0;
            },
            SegmentType::Line { .. } => {
                // println!("Checking segment from ({:?}, {:?}) to ({:?}, {:?})", self.x_start, self.y_start, self.x_end, self.y_end);
                let v1 = Vector2D::new(x - self.x_start, y - self.y_start);
                let v2 = Vector2D::new(self.x_end - self.x_start, self.y_end - self.y_start);
                let v3 = Vector2D::new(-dy, dx);

                let dot = Vector2D::dot(v2, v3);

                let mut t1 = f32::NAN;
                let mut t2 = f32::NAN;
                if dot != 0f32 {
                    t1 = cross(v2, v1) / dot;
                    t2 = Vector2D::dot(v1, v3) / dot;
                }

                let mut count = if t1 >= 0f32 && t2 >= 0f32 && t2 <= 1f32 { 1 } else { 0 };
                count = count * get_side(x, y, self.x_start, self.y_start, self.x_end, self.y_end);
                // println!("intersections with line found: {:?}", count);
                return count;
            },
            SegmentType::Quad { .. } => {
                // https://math.stackexchange.com/questions/4225469/number-of-quadratic-bezier-curve-ray-intersections
                // Now the ray we test against is the positive x-axis
                let ax = self.x_start - x;
                let ay = self.y_start - y;
                let bx = self.x1 - x;
                let by = self.y1 - y;
                let cx = self.x_end - x;
                let cy = self.y_end - y;

                // Two early-termination cases, see the link to understand these
                if ax < 0f32 && bx < 0f32 && cx < 0f32 {
                    return 0;
                } else if f32::signum(ay) == f32::signum(by) && f32::signum(by) == f32::signum(cy) {
                    return 0;
                }

                let y2 = ay - 2f32 * by + cy;
                let y1 = -2f32 * ay + 2f32 * by;
                let y0 = ay;

                let mut count: i32 = 0;
                let solns = find_roots_quadratic(y2, y1, y0);

                for &t in solns.as_ref().iter() {
                    if t >= 0f32 && t <= 1f32 {
                        // Now we check if the intersection point has x > 0, which means
                        // that there's a valid intersection in the original curve (since
                        // we originally translated to the origin)
                        let (lx, _) = quadratic_position(ax, ay, bx, by, cx, cy, t);
                        if lx > 0f32 {
                            count = count + 1;
                        }
                    }
                }
                return count;
            },
            SegmentType::Cubic { .. } => {
                // https://www.xarg.org/book/computer-graphics/line-segment-bezier-curve-intersection/
                // https://math.stackexchange.com/questions/1337440/cubic-bezier-curve-and-a-straight-line-intersection

                // Control points
                let c0_x = self.x_start - x;
                let c0_y = self.y_start - y; // A
                let c1_x = self.x1 - x;
                let c1_y = self.y1 - y; // B
                let c2_x = self.x2 - x;
                let c2_y = self.y2 - y; // C
                let c3_x = self.x_end - x;
                let c3_y = self.y_end - y; // D

                // println!("Checking Cubic Bezier with points ({:?}, {:?}), ({:?}, {:?}), ({:?}, {:?}), ({:?}, {:?})",
                //         C0_x, C0_y, C1_x, C1_y, C2_x, C2_y, C3_x, C3_y);

                let ay = -c0_y - 3f32 * c2_y + c3_y; // -A -3C + D
                let by = 3f32 * c0_y + 3f32 * c1_y + 3f32 * c2_y; // 3A + 3B + 3C
                let cy = -3f32 * c0_y - 6f32 * c1_y; // -3A - 6B
                let dy = c0_y + 3f32 * c1_y + c3_y; // A + 3B + D

                let solns = find_roots_cubic(
                    ay,
                    by,
                    cy,
                    dy
                );

                let mut count: i32 = 0;
                for &t in solns.as_ref().iter() {
                    if t >= 0f32 && t <= 1f32 {
                        // Now we check if the intersection point has x > 0, which means
                        // that there's a valid intersection in the original curve (since
                        // we originally translated to the origin)
                        let (lx, _) = cubic_position(c0_x, c0_y, c1_x, c1_y, c2_x, c2_y, c3_x, c3_y, t);
                        if lx > 0f32 {
                            count = count + 1;
                        }
                    }
                }
                // if count > 0 {
                //     println!("intersections with bezier found: {:?}", count);
                // }
                return count;

            },
            // SegmentType::Cubic { .. } => {
            //     // https://www.xarg.org/book/computer-graphics/line-segment-bezier-curve-intersection/
            //     // https://math.stackexchange.com/questions/1337440/cubic-bezier-curve-and-a-straight-line-intersection

            //     // Control points
            //     let mut c0_x = self.x_start;
            //     let mut c0_y = self.y_start;
            //     let mut c1_x = self.x1;
            //     let mut c1_y = self.y1;
            //     let mut c2_x = self.x2;
            //     let mut c2_y = self.y2;
            //     let mut c3_x = self.x_end;
            //     let mut c3_y = self.y_end;

            //     // // Convert Quadratic bezier to Cubic bezier
            //     // // https://stackoverflow.com/questions/3162645/convert-a-quadratic-bezier-to-a-cubic-one
            //     // match self.segment_type {
            //     //     SegmentType::Quad => {
            //     //         c0_x = self.x_start;
            //     //         c0_y = self.y_start;

            //     //         c1_x = self.x_start + (2./3.) * (self.x1 - self.x_start);
            //     //         c1_y = self.y_start + (2./3.) * (self.y1 - self.y_start);

            //     //         c2_x = self.x_start + (2./3.) * (self.x1 - self.x_end);
            //     //         c2_y = self.y_start + (2./3.) * (self.y1 - self.y_end);

            //     //         c3_x = self.x_end;
            //     //         c3_y = self.y_end;
            //     //     },
            //     //     _ => (),
            //     // }
            //     println!("Checking Bezier with points ({:?}, {:?}), ({:?}, {:?}), ({:?}, {:?}), ({:?}, {:?})",
            //             c0_x, c0_y, c1_x, c1_y, c2_x, c2_y, c3_x, c3_y);

            //     // Coefficients of bezier polynomial
            //     // c0_x = P0_x;
            //     // c0_y = P0_y;

            //     // c1_x = -3*P0_x + 3*P1_x;
            //     // c1_y = -3*P0_y + 3*P1_y;

            //     // c2_x = 3*P0_x - 6*P1_x + 3*P2_x;
            //     // c2_y = 3*P0_y - 6*P1_y + 3*P2_y;

            //     // c3_x = -P0_x + 3*P1_x - 3*P2_x + P3_x;
            //     // c3_y = -P0_y + 3*P1_y - 3*P2_y + P3_y;



            //     let mut count: i32 = 0;
            //     let Ax: f32 = 3f32 * (c1_x - c2_x) + c3_x - c0_x;
            //     let Ay: f32 = 3f32 * (c1_y - c2_y) + c3_y - c0_y;
                
            //     let Bx: f32 = 3f32 * (c0_x - 2f32 * c1_x + c2_x);
            //     let By: f32 = 3f32 * (c0_y - 2f32 * c1_y + c2_y);
                
            //     let Cx: f32 = 3f32 * (c1_x - c0_x);
            //     let Cy: f32 = 3f32 * (c1_y - c0_y);

            //     let Dx: f32 = c0_x;
            //     let Dy: f32 = c0_y;

            //     // const vx = B.y - A.y;
            //     // const vy = A.x - B.x;
            //     let vx: f32 = dy;
            //     let vy: f32 = -dx;

            //     // const d = A.x * vx + A.y * vy;
            //     let d: f32 = x * vx + y * vy;

            //     let solns = find_roots_cubic(
            //         vx * Ax + vy * Ay,
            //         vx * Bx + vy * By,
            //         vx * Cx + vy * Cy,
            //         vx * Dx + vy * Dy - d
            //     );

            //     // let solns = find_roots_cubic(
            //     //     vx * Dx + vy * Dy - d,
            //     //     vx * Cx + vy * Cy,
            //     //     vx * Bx + vy * By,
            //     //     vx * Ax + vy * Ay
            //     // );
            //     for &t in solns.as_ref().iter() {
            //         if t >= 0f32 && t <= 1f32 {
            //             count = count + 1;
            //         }
            //         // if 0. > t || t > 1. {
            //         //     continue;
            //         // }
            //         // count = count + 1;
            //         // res.push({
            //         //     x: ((Ax * t + Bx) * t + Cx) * t + Dx,
            //         //     y: ((Ay * t + By) * t + Cy) * t + Dy
            //         // });
            //     }
            //     if count > 0 {
            //         println!("intersections with bezier found: {:?}", count);
            //     }
            //     return count;
            // },
            SegmentType::Close { .. } => {
                return 0;
            }
        };
    }
}

// Helper functions for intersection-testing
fn get_side(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> i32 {
    /* Given a point (px, py), determines whether it is to the left or to the right of the
    plane defined by origin (x1, y1) and direction (x2, y2). Returns -1 if it is to the left,
    +1 if it is to the right*/
    // TODO: what if it overlaps with the ray?
    let side = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
    if side > 0f32 {
        return -1;
    } else if side < 0f32 {
        return 1;
    } else {
        return 0;
    }

}

fn cross(u: Vector2D<f32>, v: Vector2D<f32>) -> f32 {
    return u.x * v.y - u.y * v.x;
}

fn quadratic_position(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32, t: f32) -> (f32, f32) {
    // Returns (x, y) of quad. curve (xi, yi) at time t
    let x = f32::powi(1f32 - t, 2) * ax + 2f32 * t * (1f32 - t) * bx + f32::powi(t, 2) * cx;
    let y = f32::powi(1f32 - t, 2) * ay + 2f32 * t * (1f32 - t) * by + f32::powi(t, 2) * cy;
    return (x, y);
}

/*
fn quadratic_tangent(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32, t: f32) -> (f32, f32) {
    // Returns (dx, dy) of quad. curve (xi, yi) at time t
    let dx = 2f32 * ax * t - 2f32 * ax - 4f32 * bx * t + 2f32 * bx + 2f32 * cx * t;
    let dy = 2f32 * ay * t - 2f32 * ay - 4f32 * by * t + 2f32 * by + 2f32 * cy * t;
    return (dx, dy);
}
*/

fn cubic_position(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32, dx: f32, dy: f32, t: f32) -> (f32, f32) {
    // Returns (x, y) of cubic curve (xi, yi) at time t
    let x = f32::powi(1f32 - t, 3) * ax + 3f32 * t * f32::powi(1f32 - t, 2) * bx + 3f32 * (1f32 - t) * f32::powi(t, 2) * cx + f32::powi(t, 3) * dx;
    let y = f32::powi(1f32 - t, 3) * ay + 3f32 * t * f32::powi(1f32 - t, 2) * by + 3f32 * (1f32 - t) * f32::powi(t, 2) * cy + f32::powi(t, 3) * dy;
    return (x, y);
}