use ahash::AHashMap;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0};
use windows::core::PCWSTR;

struct FileEntry {
    name: Box<[u8]>,
    parent_frn: u64,
}

fn open_volume(drive_latter: char) -> windows::core::Result<HANDLE> {
    let path = format!("\\\\.\\{}:", drive_latter);
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }
}
fn enum_usn_data(handle: HANDLE) {
    let mut start_frn: u64 = 0;
    let mut total_records = 0u64;
    let mut buffer = vec![0u8; 1024 * 1024];

    let mut map: AHashMap<u64, FileEntry> = AHashMap::with_capacity(4_000_000);

    loop {
        let input = MFT_ENUM_DATA_V0 {
            StartFileReferenceNumber: start_frn,
            LowUsn: 0,
            HighUsn: i64::MAX,
        };

        let mut bytes_returned: u32 = 0;

        let Result = unsafe {
            DeviceIoControl(
                handle,
                FSCTL_ENUM_USN_DATA,
                Some(&input as *const _ as *const _),
                std::mem::size_of::<MFT_ENUM_DATA_V0>() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            )
        };
        match Result {
            Ok(_) => {
                start_frn = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
                let count = parse_usn_buffer(&buffer, bytes_returned, &mut map);
                total_records += count;
            }
            Err(e) if e.code() == ERROR_HANDLE_EOF.to_hresult() => {
                println!("MFT fully read.total record {total_records}");
                println!("map size: {}", map.len());
                break;
            }
            Err(e) => {
                println!("failed: {e:? } ");
                break;
            }
        }
    }
    let test_frns = [281474976710688u64, 281474976710686, 5348024557502483];

    for frn in test_frns {
        println!("{}", resolve_path(frn, &map));
    }
   
    search("main_mole_logo", &map);
  
}
fn parse_usn_buffer(buffer: &[u8], bytes_returned: u32, map: &mut AHashMap<u64, FileEntry>) -> u64 {
    let data = &buffer[..bytes_returned as usize];
    if data.len() < 8 {
        return 0;
    }

    let mut offset = 8usize;
    let mut count = 0u64;

    while offset + 60 <= data.len() {
        let record = &data[offset..];

        let record_len = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
        let frn = u64::from_le_bytes(record[8..16].try_into().unwrap());
        let parent_frn = u64::from_le_bytes(record[16..24].try_into().unwrap());
        let fname_len = u16::from_le_bytes(record[56..58].try_into().unwrap()) as usize;
        let fname_offset = u16::from_le_bytes(record[58..60].try_into().unwrap()) as usize;

        if record_len == 0 {
            break;
        }

        let name_bytes = &record[fname_offset..fname_offset + fname_len];

        let name = name_bytes.to_vec().into_boxed_slice();

        map.insert(frn, FileEntry { name, parent_frn });

        offset += record_len;
        count += 1;
    }

    count
}

fn search(query: &str, map: &AHashMap<u64, FileEntry>) {
    let query_lower = query.to_lowercase();
    let mut results = vec![];

    for (frn, entry) in map.iter() {
        let name = decode_name(&entry.name);
        if name.to_lowercase().contains(&query_lower) {
            results.push(resolve_path(*frn, map));
        }
    }
    println!("found {} results:", results.len());
    for r in &results {
        println!("  {r}");
    }
}

fn decode_name(raw: &[u8]) -> String {
    let u16s: Vec<u16> = raw
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    String::from_utf16_lossy(&u16s).into()
}

fn resolve_path(frn: u64, map: &AHashMap<u64, FileEntry>) -> String {
    let mut parts = vec![];
    let mut current_frn = frn;
    let mut depth = 0;

    loop {
        if depth > 64 {
            break;
        }
        depth += 1;

        match map.get(&current_frn) {
            Some(entry) => {
                parts.push(decode_name(&entry.name));
                current_frn = entry.parent_frn;
            }
            None => break,
        }
    }
    parts.reverse();
    format!("C:\\{}", parts.join("\\"))
}
fn main() {
    let start = std::time::Instant::now();
    match open_volume('C') {
        Ok(handle) => {
            println!("opened: {:?}", handle);
            enum_usn_data(handle);
            unsafe {
                CloseHandle(handle).ok();
            }
        }
        Err(e) => println!("failed: {e:?} (run as admin?)"),
    }
    println!("took: {:.2?}", start.elapsed());
}
