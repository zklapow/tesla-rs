use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};

use influx_db_client;
use tesla::reqwest::Error as TeslaError;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("Error writing to Influx: {}", source))]
    InfluxWrite {
        source: influx_db_client::Error
    },
    #[snafu(display("Error fetching data from Tesla API: {}", source))]
    TeslaApi {
        source: TeslaError,
    },
    #[snafu(display("Unknown vehicle state: {}", state))]
    UnknownState {
        state: String,
    }
}
