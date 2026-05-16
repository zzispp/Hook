use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};

#[derive(Clone, Debug, Default)]
pub struct TcpConnectionSnapshot {
    pub current_connections: i64,
    pub new_connections: i64,
    pub tcp_total: i64,
    pub time_wait: i64,
    pub established: i64,
    pub close_wait: i64,
}

pub fn tcp_snapshot() -> Result<TcpConnectionSnapshot, netstat2::error::Error> {
    let sockets = get_sockets_info(AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6, ProtocolFlags::TCP)?;
    Ok(sockets.into_iter().fold(TcpConnectionSnapshot::default(), |mut current, socket| {
        if let ProtocolSocketInfo::Tcp(tcp) = socket.protocol_socket_info {
            current.add(tcp.state);
        }
        current
    }))
}

impl TcpConnectionSnapshot {
    fn add(&mut self, state: TcpState) {
        self.tcp_total += 1;
        if state != TcpState::Listen {
            self.current_connections += 1;
        }
        match state {
            TcpState::SynReceived | TcpState::SynSent => self.new_connections += 1,
            TcpState::TimeWait => self.time_wait += 1,
            TcpState::Established => self.established += 1,
            TcpState::CloseWait => self.close_wait += 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use netstat2::TcpState;

    use super::TcpConnectionSnapshot;

    #[test]
    fn tcp_snapshot_counts_key_connection_states() {
        let mut snapshot = TcpConnectionSnapshot::default();
        for state in [
            TcpState::Listen,
            TcpState::Established,
            TcpState::CloseWait,
            TcpState::TimeWait,
            TcpState::SynSent,
        ] {
            snapshot.add(state);
        }

        assert_eq!(snapshot.tcp_total, 5);
        assert_eq!(snapshot.current_connections, 4);
        assert_eq!(snapshot.established, 1);
        assert_eq!(snapshot.close_wait, 1);
        assert_eq!(snapshot.time_wait, 1);
        assert_eq!(snapshot.new_connections, 1);
    }
}
