use advent_of_code_2019::Input;
use err_derive::Error;
use futures_util::stream::TryStreamExt;
use std::cmp::{max, min};
use std::error;
use std::str::FromStr;

/// 2d point
#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    /// Create new point at given coordinate
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    /// Manhattan distance of point to center
    fn distance(self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

/// 2d line
#[derive(Clone, Copy, Debug)]
struct Line {
    start: Point,
    end: Point,
}

impl Line {
    /// Create new line between given points
    fn new(start: Point, end: Point) -> Self {
        Line { start, end }
    }

    /// Left most x coordinate
    fn left(&self) -> i32 {
        min(self.start.x, self.end.x)
    }

    /// Right most x coordinate
    fn right(&self) -> i32 {
        max(self.start.x, self.end.x)
    }

    /// Bottom most y coordinate
    fn bottom(&self) -> i32 {
        min(self.start.y, self.end.y)
    }

    /// Top most y coordinate
    fn top(&self) -> i32 {
        max(self.start.y, self.end.y)
    }

    /// Length of the line (parallel to x/y axis only for now)
    fn len(&self) -> i32 {
        self.distance_to(self.end).unwrap()
    }

    /// Distance from start of line to point on line (parallel to x/y axis only)
    fn distance_to(&self, p: Point) -> Option<i32> {
        if self.start.x == self.end.x && p.x == self.start.x {
            Some((p.y - self.start.y).abs())
        } else if self.start.y == self.end.y && p.y == self.start.y {
            Some((p.x - self.start.x).abs())
        } else {
            None
        }
    }

    /// Intersection point with some other line (parallel to x/y axis and perpendicular only for now)
    fn intersection(&self, other: Line) -> Option<Point> {
        if self.start.x == self.end.x && other.start.y == other.end.y // self vertical, other horizontal
            && self.start.x > other.left() && self.start.x < other.right()
            && other.start.y > self.bottom() && other.start.y < self.top()
        {
            Some(Point::new(self.start.x, other.start.y))
        } else if self.start.y == self.end.y && other.start.x == other.end.x // self horizontal, other vertical
            && self.start.y > other.bottom() && self.start.y < other.top()
            && other.start.x > self.left() && other.start.x < self.right()
        {
            Some(Point::new(other.start.x, self.start.y))
        } else {
            None
        }
    }
}

/// Path of 2d lines
#[derive(Debug)]
struct Path {
    positions: Vec<Point>,
}

/// Error returned when a path couldn't be parsed
#[derive(Debug, Error)]
enum ParsePathError {
    #[error(display = "Invalid direction '{}'", _0)]
    InvalidDirection(String),
    #[error(display = "Invalid distance '{}'", _0)]
    InvalidDistance(String),
}

impl FromStr for Path {
    type Err = ParsePathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut last_position = Point::new(0, 0);
        let mut positions = vec![last_position];
        for step in s.split(',') {
            let distance = step[1..]
                .parse::<u32>()
                .map_err(|_e| ParsePathError::InvalidDistance(step[1..].into()))?
                as i32;
            let next_position = match &step[..1] {
                "L" => Point::new(last_position.x - distance, last_position.y),
                "R" => Point::new(last_position.x + distance, last_position.y),
                "D" => Point::new(last_position.x, last_position.y - distance),
                "U" => Point::new(last_position.x, last_position.y + distance),
                dir => return Err(ParsePathError::InvalidDirection(dir.into())),
            };
            positions.push(next_position);
            last_position = next_position;
        }
        Ok(Self { positions })
    }
}

impl Path {
    /// Iterate over lines of this path
    fn lines(&self) -> impl Iterator<Item = Line> + '_ {
        self.positions.windows(2).map(|p| Line::new(p[0], p[1]))
    }

    /// Distance to point on path
    fn distance_to(&self, p: Point) -> Option<i32> {
        let mut dist = 0;
        for line in self.lines() {
            match line.distance_to(p) {
                Some(d) => return Some(dist + d),
                None => dist += line.len(),
            }
        }
        None
    }

    /// Calculate list of intersection points between two paths
    fn intersections(&self, other: &Self) -> Vec<Point> {
        let mut res = Vec::new();
        for line1 in self.lines() {
            for line2 in other.lines() {
                if let Some(point) = line1.intersection(line2) {
                    if !(point.x == 0 && point.y == 0) {
                        res.push(point);
                    }
                }
            }
        }
        res
    }

    /// Calculate point of closest intersection
    fn closest_intersection(&self, other: &Self) -> Option<(Point, i32)> {
        self.intersections(other)
            .into_iter()
            .map(|p| (p, p.distance()))
            .min_by_key(|(_p, d)| *d)
    }

    /// Calculate point of shortest intersection
    fn shortest_intersection(&self, other: &Self) -> Option<(Point, i32)> {
        self.intersections(other)
            .into_iter()
            .map(|p| {
                (
                    p,
                    self.distance_to(p).unwrap() + other.distance_to(p).unwrap(),
                )
            })
            .min_by_key(|(_p, d)| *d)
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let mut lines = Input::day(3).await?.lines();
    let wire1 = lines.try_next().await?.unwrap().parse::<Path>()?;
    let wire2 = lines.try_next().await?.unwrap().parse::<Path>()?;
    println!(
        "Closest intersection distance: {}",
        wire1.closest_intersection(&wire2).unwrap().1,
    );

    println!(
        "Shortest intersection distance: {}",
        wire1.shortest_intersection(&wire2).unwrap().1,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_1() {
        let wire1: Path = "R8,U5,L5,D3".parse().unwrap();
        let wire2: Path = "U7,R6,D4,L4".parse().unwrap();
        assert_eq!(wire1.closest_intersection(&wire2).unwrap().1, 6);

        let wire1: Path = "R75,D30,R83,U83,L12,D49,R71,U7,L72".parse().unwrap();
        let wire2: Path = "U62,R66,U55,R34,D71,R55,D58,R83".parse().unwrap();
        assert_eq!(wire1.closest_intersection(&wire2).unwrap().1, 159);

        let wire1: Path = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51"
            .parse()
            .unwrap();
        let wire2: Path = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".parse().unwrap();
        assert_eq!(wire1.closest_intersection(&wire2).unwrap().1, 135);
    }

    #[test]
    fn part_2() {
        let wire1: Path = "R8,U5,L5,D3".parse().unwrap();
        let wire2: Path = "U7,R6,D4,L4".parse().unwrap();
        assert_eq!(wire1.shortest_intersection(&wire2).unwrap().1, 30);

        let wire1: Path = "R75,D30,R83,U83,L12,D49,R71,U7,L72".parse().unwrap();
        let wire2: Path = "U62,R66,U55,R34,D71,R55,D58,R83".parse().unwrap();
        assert_eq!(wire1.shortest_intersection(&wire2).unwrap().1, 610);

        let wire1: Path = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51"
            .parse()
            .unwrap();
        let wire2: Path = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".parse().unwrap();
        assert_eq!(wire1.shortest_intersection(&wire2).unwrap().1, 410);
    }
}
