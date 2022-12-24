use std::net;
use windows_sys::Win32::Foundation;
use windows_sys::Win32::System::SystemInformation::SetSystemTime;
use windows_sys::Win32::System::Time::FileTimeToSystemTime;

fn main() {
    let sock = net::UdpSocket::bind("0.0.0.0:0").unwrap();

    // https://lettier.github.io/posts/2016-04-26-lets-make-a-ntp-client-in-c.html
    let mut ntp_packet = [0; 48];
    ntp_packet[0] = 0x43;

    sock.send_to(&ntp_packet, "time.cloudflare.com:123")
        .unwrap();
    sock.recv_from(&mut ntp_packet).unwrap();

    //2208988800 => number of secs between NTP epoch & UNIX epochs
    let txTimeSecs = ((ntp_packet[40] as u32) << 24
        | (ntp_packet[41] as u32) << 16
        | (ntp_packet[42] as u32) << 8
        | (ntp_packet[43] as u32) - 2208988800);

    let txTimeFracSecs = (ntp_packet[44] as u32) << 24
        | (ntp_packet[45] as u32) << 16
        | (ntp_packet[46] as u32) << 8
        | (ntp_packet[47] as u32);

    // 116444736000000000 ==> Number of 100 ns intervals between Win32 epochs & UNIX epochs
    // https://en.wikipedia.org/wiki/Epoch_(computing)
    let mut ns_intervals: u64 = 116444736000000000;
    ns_intervals += txTimeSecs as u64 * (1_000_000_000 / 100);
    ns_intervals += (txTimeFracSecs as u64 * 1_000_000 * 10) / 4294967296;

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
            systemtime.wYear,
            systemtime.wMonth,
            systemtime.wDay,
            systemtime.wHour,
            systemtime.wMinute,
            systemtime.wSecond,
            systemtime.wMilliseconds
        );
    }
}
