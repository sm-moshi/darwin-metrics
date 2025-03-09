use std::time::Instant;

/// Represents a network traffic data point with received and sent data.
#[derive(Debug, Clone, Copy)]
pub struct TrafficData {
    /// Time when this data point was collected
    pub timestamp: Instant,

    /// Total bytes received
    pub bytes_received: u64,

    /// Total bytes sent
    pub bytes_sent: u64,

    /// Total packets received
    pub packets_received: u64,

    /// Total packets sent
    pub packets_sent: u64,

    /// Total receive errors
    pub receive_errors: u64,

    /// Total send errors
    pub send_errors: u64,

    /// Total collisions
    pub collisions: u64,
}

impl TrafficData {
    /// Creates a new TrafficData instance with the given metrics.
    pub fn new(
        bytes_received: u64,
        bytes_sent: u64,
        packets_received: u64,
        packets_sent: u64,
        receive_errors: u64,
        send_errors: u64,
        collisions: u64,
    ) -> Self {
        Self {
            timestamp: Instant::now(),
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
            receive_errors,
            send_errors,
            collisions,
        }
    }
}

/// Tracks network traffic statistics over time and calculates rates.
#[derive(Debug, Clone)]
pub struct TrafficTracker {
    /// Current network traffic data
    current: TrafficData,

    /// Previous network traffic data for rate calculations
    previous: Option<TrafficData>,
}

impl TrafficTracker {
    /// Creates a new TrafficTracker with initial values.
    pub fn new(
        bytes_received: u64,
        bytes_sent: u64,
        packets_received: u64,
        packets_sent: u64,
        receive_errors: u64,
        send_errors: u64,
        collisions: u64,
    ) -> Self {
        let current = TrafficData::new(
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
            receive_errors,
            send_errors,
            collisions,
        );

        Self { current, previous: None }
    }

    /// Updates the traffic data and shifts current data to previous.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        bytes_received: u64,
        bytes_sent: u64,
        packets_received: u64,
        packets_sent: u64,
        receive_errors: u64,
        send_errors: u64,
        collisions: u64,
    ) {
        self.previous = Some(self.current);
        self.current = TrafficData::new(
            bytes_received,
            bytes_sent,
            packets_received,
            packets_sent,
            receive_errors,
            send_errors,
            collisions,
        );
    }

    /// Gets the current bytes received count.
    pub fn bytes_received(&self) -> u64 {
        self.current.bytes_received
    }

    /// Gets the current bytes sent count.
    pub fn bytes_sent(&self) -> u64 {
        self.current.bytes_sent
    }

    /// Gets the current packets received count.
    pub fn packets_received(&self) -> u64 {
        self.current.packets_received
    }

    /// Gets the current packets sent count.
    pub fn packets_sent(&self) -> u64 {
        self.current.packets_sent
    }

    /// Gets the current receive errors count.
    pub fn receive_errors(&self) -> u64 {
        self.current.receive_errors
    }

    /// Gets the current send errors count.
    pub fn send_errors(&self) -> u64 {
        self.current.send_errors
    }

    /// Gets the current collisions count.
    pub fn collisions(&self) -> u64 {
        self.current.collisions
    }

    /// Calculates the current download speed in bytes per second.
    /// Returns 0.0 if there's no previous data point for comparison.
    pub fn download_speed(&self) -> f64 {
        if let Some(prev) = self.previous {
            let bytes_diff = self.current.bytes_received.saturating_sub(prev.bytes_received);
            let time_diff = self.current.timestamp.duration_since(prev.timestamp).as_secs_f64();

            if time_diff > 0.0 {
                bytes_diff as f64 / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculates the current upload speed in bytes per second.
    /// Returns 0.0 if there's no previous data point for comparison.
    pub fn upload_speed(&self) -> f64 {
        if let Some(prev) = self.previous {
            let bytes_diff = self.current.bytes_sent.saturating_sub(prev.bytes_sent);
            let time_diff = self.current.timestamp.duration_since(prev.timestamp).as_secs_f64();

            if time_diff > 0.0 {
                bytes_diff as f64 / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculates the packet receive rate (packets per second).
    pub fn packet_receive_rate(&self) -> f64 {
        if let Some(prev) = self.previous {
            let packets_diff = self.current.packets_received.saturating_sub(prev.packets_received);
            let time_diff = self.current.timestamp.duration_since(prev.timestamp).as_secs_f64();

            if time_diff > 0.0 {
                packets_diff as f64 / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculates the packet send rate (packets per second).
    pub fn packet_send_rate(&self) -> f64 {
        if let Some(prev) = self.previous {
            let packets_diff = self.current.packets_sent.saturating_sub(prev.packets_sent);
            let time_diff = self.current.timestamp.duration_since(prev.timestamp).as_secs_f64();

            if time_diff > 0.0 {
                packets_diff as f64 / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculates the error rate for received packets.
    pub fn receive_error_rate(&self) -> f64 {
        if self.current.packets_received > 0 {
            self.current.receive_errors as f64 / self.current.packets_received as f64
        } else {
            0.0
        }
    }

    /// Calculates the error rate for sent packets.
    pub fn send_error_rate(&self) -> f64 {
        if self.current.packets_sent > 0 {
            self.current.send_errors as f64 / self.current.packets_sent as f64
        } else {
            0.0
        }
    }
}
