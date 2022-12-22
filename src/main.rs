use sntpc::{Error, NtpContext, NtpTimestampGenerator, NtpUdpSocket, Result};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, SystemTime};

use windows_sys::Win32::Foundation;
use windows_sys::Win32::System::SystemInformation::SetSystemTime;
use windows_sys::Win32::System::Time::FileTimeToSystemTime;

#[derive(Copy, Clone, Default)]
struct StdTimestampGen {
    duration: Duration,
}

impl NtpTimestampGenerator for StdTimestampGen {
    fn init(&mut self) {
        self.duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
    }

    fn timestamp_sec(&self) -> u64 {
        // println!("timestamp_sec= {}", self.duration.as_secs());
        self.duration.as_secs()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        // println!("timestamp_subsec_micros= {}", self.duration.subsec_micros());
        self.duration.subsec_micros()
    }
}

#[derive(Debug)]
struct UdpSocketWrapper {
    socket: UdpSocket,
}

impl NtpUdpSocket for UdpSocketWrapper {
    fn send_to<T: ToSocketAddrs>(&self, buf: &[u8], addr: T) -> Result<usize> {
        match self.socket.send_to(buf, addr) {
            Ok(usize) => Ok(usize),
            Err(e) => {
                println!("send_to err {}", e.to_string());
                Err(Error::Network)
            }
        }
    }

    fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        match self.socket.recv_from(buf) {
            Ok((size, addr)) => Ok((size, addr)),
            Err(e) => {
                println!("recv_from err {}", e.to_string());
                Err(Error::Network)
            }
        }
    }
}

fn set_time(sec: u32, msec: u32) {
    // println!("sec={} msec={}", sec, msec);

    //https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime
    // Number of 100 ns intervals since Jan 1, 1601 (UTC) to UNIX epochs
    let mut ns_intervals = 116444736000000000u64;

    ns_intervals += sec as u64 * (1_000_000_000 / 100); // 1 sec = 10e9 ns

    // http://thompsonng.blogspot.com/2010/04/ntp-timestamp_21.html
    ns_intervals += (msec as u64 * 1_000_000 * 10)/4294967296; // 1 sec = 10e6 ms, 4294967296 == 2**32

    let filetime = Foundation::FILETIME {
        dwLowDateTime: (ns_intervals & 0xffffffff) as u32,
        dwHighDateTime: (ns_intervals >> 32) as u32,
    };

    let mut systemtime = Foundation::SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };
    unsafe {
        FileTimeToSystemTime(&filetime, &mut systemtime);
        SetSystemTime(&systemtime);
        println!(
            "[+] Time set to\n year={} month={} day={} hour={} min={} sec={} ms={}",
            systemtime.wYear, systemtime.wMonth, systemtime.wDay, systemtime.wHour, systemtime.wMinute, systemtime.wSecond, systemtime.wMilliseconds
        );
    }
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Unable to create UDP socket");

    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Unable to set UDP socket read timeout");

    let sock_wrapper = UdpSocketWrapper { socket };
    let ntp_context = NtpContext::new(StdTimestampGen::default());

    println!("[+] Querying time");
    let result = sntpc::get_time("time.cloudflare.com:123", sock_wrapper, ntp_context);

    match result {
        Ok(time) => {
            set_time(time.sec(), time.sec_fraction());
        }
        Err(_) => println!("!Error"),
    }
}
