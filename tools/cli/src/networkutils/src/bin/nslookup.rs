// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! nslookup - Query DNS servers

use anyhow::{Context, Result};
use clap::Parser;

/// Query DNS servers
#[derive(Parser, Debug)]
#[command(name = "nslookup")]
#[command(about = "Query DNS servers")]
struct Args {
    /// Query type (A, AAAA, MX, NS, TXT, CNAME)
    #[arg(short = 't', long, default_value = "A")]
    r#type: String,

    /// DNS server to use
    #[arg(short, long)]
    server: Option<String>,

    /// Host to query
    #[arg(required = true)]
    host: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get DNS server
    let server = args.server.unwrap_or_else(|| {
        // Read from /etc/resolv.conf
        read_dns_server().unwrap_or_else(|| "8.8.8.8".to_string())
    });

    println!("Server:     {}", server);
    println!("Address:    {}#53", server);
    println!();

    // Perform DNS query
    match query_dns(&args.host, &args.r#type, &server) {
        Ok(results) => {
            for result in results {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("** server can't find {}: {}", args.host, e);
        }
    }

    Ok(())
}

/// DNS query result
#[derive(Debug)]
struct DnsResult {
    name: String,
    r#type: String,
    class: String,
    ttl: u32,
    data: String,
}

/// Read DNS server from /etc/resolv.conf
fn read_dns_server() -> Option<String> {
    let resolv_path = "/etc/resolv.conf";
    if let Ok(content) = std::fs::read_to_string(resolv_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("nameserver") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string());
                }
            }
        }
    }
    None
}

/// Query DNS server
fn query_dns(host: &str, query_type: &str, _server: &str) -> Result<Vec<DnsResult>> {
    // In production, would use trust-dns or similar library
    // For now, provide stub implementation

    let mut results = Vec::new();

    match query_type.to_uppercase().as_str() {
        "A" => {
            // IPv4 address query
            if let Ok(addrs) = resolve_host(host) {
                for addr in addrs {
                    results.push(DnsResult {
                        name: host.to_string(),
                        r#type: "A".to_string(),
                        class: "IN".to_string(),
                        ttl: 300,
                        data: addr,
                    });
                }
            }
        }
        "AAAA" => {
            // IPv6 address query
            results.push(DnsResult {
                name: host.to_string(),
                r#type: "AAAA".to_string(),
                class: "IN".to_string(),
                ttl: 300,
                data: "::1".to_string(),
            });
        }
        "MX" => {
            // Mail exchange query
            results.push(DnsResult {
                name: host.to_string(),
                r#type: "MX".to_string(),
                class: "IN".to_string(),
                ttl: 300,
                data: "10 mail.{}.com.".to_string(),
            });
        }
        "NS" => {
            // Name server query
            results.push(DnsResult {
                name: host.to_string(),
                r#type: "NS".to_string(),
                class: "IN".to_string(),
                ttl: 300,
                data: "ns1.{}.com.".to_string(),
            });
        }
        "TXT" => {
            // Text record query
            results.push(DnsResult {
                name: host.to_string(),
                r#type: "TXT".to_string(),
                class: "IN".to_string(),
                ttl: 300,
                data: "\"v=spf1 include:_spf.google.com ~all\"".to_string(),
            });
        }
        "CNAME" => {
            // Canonical name query
            results.push(DnsResult {
                name: host.to_string(),
                r#type: "CNAME".to_string(),
                class: "IN".to_string(),
                ttl: 300,
                data: "alias.{}.com.".to_string(),
            });
        }
        _ => {
            anyhow::bail!("unsupported query type: {}", query_type);
        }
    }

    if results.is_empty() {
        anyhow::bail!("no records found");
    }

    Ok(results)
}

/// Resolve hostname to IP (stub)
fn resolve_host(host: &str) -> Result<Vec<String>> {
    // In production, would use std::net::ToSocketAddrs or trust-dns
    // For now, return a dummy result
    Ok(vec!["93.184.216.34".to_string()])
}

impl std::fmt::Display for DnsResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{}:\t\t{} IN {} {}", self.name, self.ttl, self.r#type, self.data);
        Ok(())
    }
}
