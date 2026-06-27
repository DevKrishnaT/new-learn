use std::collections::HashMap;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Ioctl::{FSCTL_ENUM_USN_DATA, MFT_ENUM_DATA_V0};
use windows::core::PCWSTR;

struct FileEntry {
    name: String,
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
    let mut buffer = vec![0u8; 64 * 1024];
    let mut map: HashMap<u64, FileEntry> = HashMap::with_capacity(4_000_000);

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
                let count = parse_usn_buffer(&buffer, bytes_returned , &mut map);
                total_records += count;
                println!("--- batch done, total so far: {total_records} ---");
            }
            Err(e) if e.code() == ERROR_HANDLE_EOF.to_hresult() => {
                println!("MFT fully read.total record {total_records}");
                println!("map size: {}", map.len());
                break;
            }
            Err(e) => {
                println!("failed: {e:?}");
                break;
            }
        }
    }
}
fn parse_usn_buffer(buffer: &[u8], bytes_returned: u32 , map: &mut HashMap <u64 , FileEntry>) -> u64 {
    let data = &buffer[..bytes_returned as usize];
    if data.len() < 8 {
        return 0;
    }
    let next_frn = u64::from_le_bytes(data[0..8].try_into().unwrap());
    println!("next cursor frn: {next_frn}");

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
        let name_u16: Vec<u16> = name_bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        let name = String::from_utf16_lossy(&name_u16);

        map.insert(frn, FileEntry { name, parent_frn });

        offset += record_len;
        count += 1;
    }
    count
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
