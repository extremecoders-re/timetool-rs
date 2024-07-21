use std::net::{self, IpAddr, Ipv4Addr, ToSocketAddrs};
use windows_sys::Win32::Foundation;
use windows_sys::Win32::System::SystemInformation::SetSystemTime;
use windows_sys::Win32::System::Time::FileTimeToSystemTime;

// We need to filter for IPv4 address from DNS lookup as otherwise the IP address 
// of the NTP domain can resolve to a v6 address which won't work with a UDP socket bound
// on v4 
fn dnslookup_ipv4(domain: &str) -> Option<Ipv4Addr> {
    let addrs_iter = (domain, 0).to_socket_addrs().expect("Failed to resolve domain");

    // Filter out non-IPv4 addresses
    for addr in addrs_iter {
        if let IpAddr::V4(ipv4_addr) = addr.ip() {
            return Some(ipv4_addr);
        }
    }
    None
}

fn main() {
    // Bind on v4 only
    let sock = net::UdpSocket::bind("0.0.0.0:0").unwrap();

    // https://lettier.github.io/posts/2016-04-26-lets-make-a-ntp-client-in-c.html
    let mut ntp_packet = [0; 48];
    /*
    Breakdown of li_vn_mode

    Leap Indicator (LI): The first two bits of this field are reserved for the leap indicator.
    These bits indicate whether an impending leap second should be inserted or deleted in the last minute of the current day.
    The values can represent:
        No warning (00)
        Last minute has 61 seconds (01)
        Last minute has 59 seconds (10)
        Alarm condition (clock unsynchronized) (11)

    Version Number (VN): The next three bits after the LI are used for the version number.
    This indicates the version of the protocol being used.
    For example, a value of 001 would indicate NTPv3, while 010 would indicate NTPv4.

    Mode: The remaining five bits in this octet specify the mode of operation.
    The mode determines the role of the sender, such as client, server, peer, etc.
    Some common modes include:
        Reserved (000)
        Symmetric active (001)
        Symmetric passive (010)
        Client (011)
        Server (100)
        Broadcast (101)
        NTP control message (110)
        Private use (111)
    */

    // li_vn_mode (LI=00 VN=010 Mode=011)
    ntp_packet[0] = 0b00_010_011;

    // https://gist.github.com/mutin-sa/eea1c396b1e610a2da1e5550d94b0453
    const NTP_SERVER_ADDR: &str = "time.cloudflare.com";
    println!("[+] Resolving DNS: {}", NTP_SERVER_ADDR);

    let ntp_server_ipv4 = dnslookup_ipv4(NTP_SERVER_ADDR).unwrap();
    println!("[+] Querying NTP server: {}", ntp_server_ipv4);

    sock.send_to(&ntp_packet, (ntp_server_ipv4, 123)).unwrap();
    sock.recv_from(&mut ntp_packet).unwrap();

    //2208988800 => number of secs between NTP epoch & UNIX epochs
    let tx_time_secs = ((ntp_packet[40] as u32) << 24
        | (ntp_packet[41] as u32) << 16
        | (ntp_packet[42] as u32) << 8
        | (ntp_packet[43] as u32))
        - 2208988800;

    let tx_time_frac_secs = (ntp_packet[44] as u32) << 24
        | (ntp_packet[45] as u32) << 16
        | (ntp_packet[46] as u32) << 8
        | (ntp_packet[47] as u32);

    // 116444736000000000 ==> Number of 100 ns intervals between Win32 epochs & UNIX epochs
    // https://en.wikipedia.org/wiki/Epoch_(computing)
    let mut ns_intervals: u64 = 116444736000000000;
    ns_intervals += tx_time_secs as u64 * (1_000_000_000 / 100);
    ns_intervals += (tx_time_frac_secs as u64 * 1_000_000 * 10) / 4294967296;

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
            "[+] Time set to: year={} month={} day={} hour={} min={} sec={} ms={}",
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
