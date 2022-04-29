use std::ffi::OsString;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io::Error as IoError;

#[derive(Debug, Clone)]
pub struct StorageDevice {
	pub name: String,
	pub capacity: usize,
	pub partitions: Vec<Partition>,
	pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct Partition {
	pub name: String,
	pub capacity: usize,
	pub read_only: bool,
}

fn read_prop(base: &str, prop: &str) -> Option<String> {
	let value = read_to_string(format!("{}/{}", base, prop)).ok()?;
	Some(String::from(&value[..value.len() - 1]))
}

fn read_partition(device: &str, entry: OsString, sector_size: usize) -> Option<Partition> {
	let name = entry.into_string().ok()?;
	let path = format!("/sys/block/{}/{}", device, name);
	let sectors = read_prop(&path, "size")?.parse::<usize>().ok()?;
	let read_only = read_prop(&path, "ro")? == "1";
	Some(Partition {
		name,
		capacity: sectors * sector_size,
		read_only,
	})
}

fn read_device(entry: OsString) -> Option<StorageDevice> {
	let name = entry.into_string().ok()?;
	let path = format!("/sys/block/{}", name);
	let sectors = read_prop(&path, "size")?.parse::<usize>().ok()?;
	let sector_size = read_prop(&path, "queue/hw_sector_size")?.parse().ok()?;
	let read_only = read_prop(&path, "ro")? == "1";
	let mut partitions = Vec::new();
	for entry in read_dir(&path).ok()? {
		if let Some(partition) = read_partition(&name, entry.ok()?.file_name(), sector_size) {
			partitions.push(partition);
		}
	}
	Some(StorageDevice {
		name,
		capacity: sectors * sector_size,
		partitions,
		read_only,
	})
}

pub fn list_partitions() -> Result<Vec<StorageDevice>, IoError> {
	let mut devices = Vec::new();
	for entry in read_dir("/sys/block")? {
		if let Some(device) = read_device(entry?.file_name()) {
			devices.push(device);
		}
	}
	Ok(devices)
}
