use std::{
    io::Read,
    net::{IpAddr, ToSocketAddrs},
};
pub fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && a != 0
                && a < 224
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 198 && (18..=19).contains(&b))
                && !(a == 192 && b == 0 && c == 0)
        }
        IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            if let Some(v4) = ip.to_ipv4_mapped() {
                return public_address(IpAddr::V4(v4));
            }
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && (first & 0xfe00) != 0xfc00
                && (first & 0xffc0) != 0xfe80
                && !(first == 0x2001 && ip.segments()[1] == 0xdb8)
                && (first & 0xe000) == 0x2000
        }
    }
}
pub fn fetch(raw: &str) -> Result<(Vec<u8>, String, String), String> {
    let started = std::time::Instant::now();
    let mut url = url::Url::parse(raw).map_err(|_| "Enter a valid public HTTP(S) URL.")?;
    for _ in 0..5 {
        let remaining = std::time::Duration::from_secs(10)
            .checked_sub(started.elapsed())
            .ok_or("Source acquisition exceeded 10 seconds.")?;
        if !["http", "https"].contains(&url.scheme())
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Use a public HTTP(S) URL without credentials.".into());
        }
        let host = url.host_str().ok_or("URL host is missing.")?;
        let port = url.port_or_known_default().ok_or("URL port is missing.")?;
        let addresses = (host, port)
            .to_socket_addrs()
            .map_err(|_| "Source host could not be resolved.")?
            .collect::<Vec<_>>();
        if addresses.is_empty() || addresses.iter().any(|a| !public_address(a.ip())) {
            return Err("URL resolves to a private or reserved address. Paste or upload the source instead.".into());
        }
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .resolve_to_addrs(host, &addresses)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(remaining)
            .build()
            .map_err(|_| "Source transport unavailable.")?;
        let response = client
            .get(url.as_str())
            .header("User-Agent", "CowWorker/0.2 source-import")
            .send()
            .map_err(|_| "Source request failed. Paste or upload the source instead.")?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or("Redirect has no valid destination.")?;
            url = url.join(location).map_err(|_| "Invalid source redirect.")?;
            continue;
        }
        if !response.status().is_success() {
            return Err(format!(
                "Source returned HTTP {}. Paste or upload instead.",
                response.status().as_u16()
            ));
        }
        let media = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .split(';')
            .next()
            .unwrap_or("text/html")
            .to_string();
        if response.content_length().is_some_and(|n| n > 25_000_000) {
            return Err("Source exceeds 25 MB.".into());
        }
        let mut bytes = vec![];
        response
            .take(25_000_001)
            .read_to_end(&mut bytes)
            .map_err(|_| "Source download interrupted.")?;
        if bytes.len() > 25_000_000 {
            return Err("Source exceeds 25 MB.".into());
        }
        return Ok((bytes, media, url.to_string()));
    }
    Err("Source has too many redirects. Paste or upload instead.".into())
}
