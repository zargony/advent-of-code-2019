use advent_of_code_2019::Input;
use async_std::io;
use async_std::prelude::*;
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;
use std::marker::Unpin;

struct OrbitMap {
    /// Maps objects to the object they orbit around
    orbits: HashMap<String, String>,
}

impl OrbitMap {
    const COM: &'static str = "COM";

    /// Parse orbit map from stream of lines
    async fn load(mut lines: impl Stream<Item = io::Result<String>> + Unpin) -> io::Result<Self> {
        let mut orbits = HashMap::new();
        while let Some(line) = lines.try_next().await? {
            let mut objects = line.split(")");
            let center = objects.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing center object name")
            })?;
            let object = objects.next().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing orbiting object name")
            })?;
            orbits.insert(object.to_string(), center.to_string());
        }
        Ok(Self { orbits })
    }

    /// Returns an iterator over all objects
    fn objects(&self) -> impl Iterator<Item = &str> {
        self.orbits.keys().map(AsRef::as_ref)
    }

    /// Returns the shortest path for object to reach a destination orbit
    fn find_path<'a>(&'a self, object: &'a str, destination: &'a str) -> Vec<&'a str> {
        if destination == Self::COM {
            let mut path = vec![object];
            while let Some(parent) = self.orbits.get(*path.last().unwrap()) {
                path.push(parent);
            }
            assert!(
                path.last().unwrap() == &Self::COM,
                "Data inconsistency: object `{}` used as root",
                path.last().unwrap(),
            );
            path
        } else {
            let mut path1 = self.find_path(object, Self::COM);
            let mut path2 = self.find_path(destination, Self::COM);
            while path1.len() > 1
                && path2.len() > 1
                && path1[path1.len() - 2] == path2[path2.len() - 2]
            {
                path1.pop();
                path2.pop();
            }
            assert!(!path1.is_empty() && !path2.is_empty() && path1.last() == path2.last());
            path1.pop();
            path1.extend(path2.into_iter().rev());
            path1
        }
    }

    /// Returns the number of orbits for the given object
    fn count_orbits(&self, object: &str) -> usize {
        self.find_path(object, Self::COM).len() - 1
    }

    /// Returns the number of total orbits
    fn total_orbits(&self) -> usize {
        self.objects().map(|obj| self.count_orbits(obj)).sum()
    }

    /// Returns number of orbital transfers of the given object to the destination
    fn orbital_transfers(&self, object: &str, destination: &str) -> usize {
        self.find_path(object, destination).len() - 3
    }
}

#[async_std::main]
async fn main() -> io::Result<()> {
    let lines = Input::day(6).await?.lines();
    let orbit_map = OrbitMap::load(lines).await?;

    println!("Total number of orbits: {}", orbit_map.total_orbits());

    println!(
        "Orbital transfers of YOU to SAN: {}",
        orbit_map.orbital_transfers("YOU", "SAN"),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::stream;

    #[async_std::test]
    async fn part_1() {
        let lines = [
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
        ];
        let lines = stream::from_iter(lines.iter()).map(|l| io::Result::Ok(l.to_string()));
        let orbit_map = OrbitMap::load(lines).await.unwrap();

        assert_eq!(orbit_map.find_path("D", "COM"), ["D", "C", "B", "COM"]);
        assert_eq!(orbit_map.count_orbits("D"), 3);

        assert_eq!(
            orbit_map.find_path("L", "COM"),
            ["L", "K", "J", "E", "D", "C", "B", "COM"],
        );
        assert_eq!(orbit_map.count_orbits("L"), 7);

        assert_eq!(orbit_map.find_path("COM", "COM"), ["COM"]);
        assert_eq!(orbit_map.count_orbits("COM"), 0);
    }

    #[async_std::test]
    async fn part_2() {
        let lines = [
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
            "I)SAN",
        ];
        let lines = stream::from_iter(lines.iter()).map(|l| io::Result::Ok(l.to_string()));
        let orbit_map = OrbitMap::load(lines).await.unwrap();

        assert_eq!(
            orbit_map.find_path("YOU", "SAN"),
            ["YOU", "K", "J", "E", "D", "I", "SAN"],
        );
        assert_eq!(orbit_map.orbital_transfers("YOU", "SAN"), 4);
    }
}
